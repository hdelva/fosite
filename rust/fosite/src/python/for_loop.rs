use core::*;

use core::Path;
use std::collections::btree_map::Entry;
use std::collections::BTreeSet;
use std::collections::BTreeMap;

pub struct PythonFor { }

impl ForEachExecutor for PythonFor {
    fn execute(&self,
               env: Environment,
               before: &GastNode,
               body: &GastNode)
               -> ExecutionResult {
        let Environment { vm, executors } = env;

        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();

        vm.start_watch();
        let test_result = vm.execute(executors, before);
        vm.toggle_watch();

        for change in test_result.changes.into_iter() {
            total_changes.push(change);
        }

        for dependency in test_result.dependencies.into_iter() {
            total_dependencies.push(dependency);
        }

        // register this node as a branch
        let id = vm.current_node().clone();
        vm.push_branch(id);


        let result = self.branch(vm, executors, body, total_changes, total_dependencies);

        // register this node as a branch
        vm.pop_branch();

        return result;
    }
}

impl PythonFor {
    fn branch(&self,
              vm: &mut VirtualMachine,
              executors: &Executors,
              body: &GastNode,
              c: Vec<AnalysisItem>,
              d: Vec<AnalysisItem>) -> ExecutionResult {
                          
        let mut total_changes = c;
        let mut total_dependencies = d;

        let mut positive;
        let mut negative;
        {
            let current_path = vm.current_path();

            positive = current_path.clone();
            positive.add_node(PathNode::Loop(vm.current_node(), true));
            negative = current_path.clone();
            negative.add_node(PathNode::Loop(vm.current_node(), false));
        }

        vm.push_path(positive);
        let body_result = vm.execute(executors, body);
        let _ = vm.pop_path();

        let mut changes = body_result.changes;
        let mut dependencies = body_result.dependencies;

        total_changes.append(&mut changes);
        total_dependencies.append(&mut dependencies);

        vm.change_branch(&total_changes);

        self.check_changes(vm, &total_changes);

        // labels all changes made with the Loop id
        vm.merge_branches(&total_changes);

        self.check_types(vm, executors, &total_changes);

        return ExecutionResult {
            changes: total_changes,
            dependencies: total_dependencies,
            flow: FlowControl::Continue,
            result: Mapping::new(),
        };
    }

    fn check_changes(&self, vm: &mut VirtualMachine, changes: &Vec<AnalysisItem>) {
        // we might need to prune some paths
        let current_node = vm.current_node();

        let watch = vm.pop_watch();
        let Watch {before, after, ..} = watch;

        let mut relevant_objects = BTreeSet::new();
        let mut changed_objects = BTreeMap::new();

        for (_, mapping) in before.iter() {
            for (_, address) in mapping.iter() {
                relevant_objects.insert(address.clone());
            } 
        }



        if before.len() == 0 {
            //todo warning, trivial loop condition
        } else {
            let (identifier, mapping)  = before.iter().next().unwrap();

            let new_mapping = after.get(&identifier).unwrap();

            // initialise an empty path as a problem path
            // will get expanded everytime a variable is unchanged
            let mut problems = vec!(Path::empty());

            // because the initial problem path is empty 
            // it's only an actual problem if things get merged into it
            let mut real_problem = false;

            for (_, address) in mapping.iter() {
                // me too thanks
                let mut same = Vec::new();
                
                for (new_path, new_address) in new_mapping.iter() {
                    if address == new_address {
                        if let Some(paths) = changed_objects.get(address) {
                            // address hasn't changed 
                            // but the object at that address has changed under some conditions 
                            let mut invariants = possible_invariants(new_path, paths);
                            if invariants.len() > 0 {
                                real_problem = true;
                            }
                            same.append(&mut invariants);
                        } else {
                            // address hasn't changed 
                            // object at that address hasn't either
                            real_problem = true;
                            same.push(new_path.clone());
                        }
                    }
                }

                let mut new_problems = Vec::new();

                for problem in problems.into_iter() {
                    for new_problem in same.iter() {
                        if problem.mergeable(new_problem) {
                            let mut new = problem.clone();
                            new.merge_into(new_problem.clone());
                            new_problems.push(new);
                        }
                    }
                }

                problems = new_problems;
            }

            if real_problem {
                let content = WhileLoopChange::new(problems);
                let message = Message::Output {
                    source: vm.current_node(),
                    content: Box::new(content),
                };
                &CHANNEL.publish(message);
            }
        }
    }

    fn check_types(&self,
             vm: &mut VirtualMachine,
             executors: &Executors,
             changes: &Vec<AnalysisItem>) {
        for change in changes {
            if !change.is_object() {
                let mut all_types = BTreeMap::new();

                let execution_result = match change {
                    &AnalysisItem::Identifier { ref name } => vm.load_identifier(executors, name),
                    &AnalysisItem::Attribute { ref parent, ref name } => {
                        vm.load_attribute(executors, &parent.as_node(), name)
                    }
                    _ => {
                        unreachable!("AnalysisItem is an object when a previous check should've \
                                      excluded this")
                    }
                };

                let result = execution_result.result;
                for (path, address) in result.iter() {
                    let object = vm.get_object(address);
                    let type_name = object.get_type_name(vm.knowledge());

                    match all_types.entry(type_name.clone()) {
                        Entry::Vacant(v) => {
                            v.insert(vec![path.clone()]);
                        }
                        Entry::Occupied(mut o) => {
                            o.get_mut().push(path.clone());
                        }
                    };
                }

                if all_types.len() > 1 {
                    let content = TypeUnsafe::new(change.to_string(), all_types);
                    let message = Message::Output { 
                        source: vm.current_node(),
                        content: Box::new(content),
                    };
                    &CHANNEL.publish(message);
                }
            }
        }
    }
}

fn possible_invariants(parent_path: &Path, changes: &Vec<Path>) -> Vec<Path> {
    // remove obsolete entries first, i.e.
    // (1, 5) and (1, 5, 9)
    let mut all_reversals = BTreeSet::new();

    for change in changes {
        let change_reversals = change.reverse();
        for reversal in change_reversals.iter().rev() {
            all_reversals.insert(reversal.clone());
        }
    }

    let mut possibilities = Vec::new();
    for reversal in all_reversals.into_iter() {
        let mut legit = true;
        for change in changes {
            if reversal.contains(change) {
                legit = false;
                break;
            }
        }

        if legit && parent_path.mergeable(&reversal) {
            let mut new_path = parent_path.clone();
            new_path.merge_into(reversal);
            possibilities.push(new_path);
        }
    }

    return possibilities;
}