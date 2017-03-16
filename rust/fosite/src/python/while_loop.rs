use core::*;

use core::Path;
use std::collections::btree_map::Entry;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use std::collections::HashSet;

pub struct PythonWhile { }

impl WhileExecutor for PythonWhile {
    fn execute(&self,
               env: Environment,
               test: &GastNode,
               body: &GastNode)
               -> ExecutionResult {
        let Environment { vm, executors } = env;

        // register this node as a branch
        let id = vm.current_node().clone();
        vm.push_branch(id);

        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();

        vm.start_watch();
        let test_result = vm.execute(executors, test);
        vm.toggle_watch();

        let mut no = Vec::new();

        for change in test_result.changes.into_iter() {
            total_changes.push(change);
        }

        for dependency in test_result.dependencies.into_iter() {
            total_dependencies.push(dependency);
        }

        let f = vm.knowledge().constant(&"False".to_owned());

        // split up the test result into yes/no/maybe
        for (path, address) in test_result.result.into_iter() {
            if address == f {
                no.push(path);
            }
        }

        let result = self.branch(vm, executors, body, no, total_changes, total_dependencies);

        // register this node as a branch
        vm.pop_branch();

        return result;
    }
}

impl PythonWhile {
    fn branch(&self,
              vm: &mut VirtualMachine,
              executors: &Executors,
              body: &GastNode,
              no: Vec<Path>,
              c: Vec<AnalysisItem>,
              d: Vec<AnalysisItem>) -> ExecutionResult {
                          
        let mut total_changes = c;
        let mut total_dependencies = d;

        let mut positive;
        let mut negative;
        {
            let current_path = vm.current_path();

            positive = current_path.clone();
            positive.add_node(PathNode::Loop(vm.current_node().clone(), 0, 2));
            negative = current_path.clone();
            negative.add_node(PathNode::Loop(vm.current_node().clone(), 1, 2));
        }

        vm.push_path(positive);
        vm.add_restrictions(no);
        let body_result = vm.execute(executors, body);
        vm.drop_restrictions();
        let _ = vm.pop_path();

        let mut changes = body_result.changes;
        let mut dependencies = body_result.dependencies;

        total_changes.append(&mut changes);
        total_dependencies.append(&mut dependencies);

        vm.next_branch(&total_changes);

        self.check_changes(vm);

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

    fn check_changes(&self, vm: &mut VirtualMachine) {
        let watch = vm.pop_watch();

        // initialise an empty path as a problem path
        // will get expanded everytime a variable is unchanged
        let mut problems = vec!(Path::empty());

        // because the initial problem path is empty 
        // it's only an actual problem if things get merged into it
        let mut real_problem = false;

        for (identifier, addresses) in watch.identifiers_before.into_iter() {
            // me too thanks
            let mut same = BTreeSet::new();

            let opt_mapping = watch.identifiers_changed.get(&identifier);

            let identifier_invariants;
            if let Some(new_mapping) = opt_mapping {
                identifier_invariants = possible_identifier_invariants(&addresses, new_mapping);
            } else {
                let mut pls = BTreeMap::new();
                for address in addresses {
                    let mut pls2 = BTreeSet::new();
                    pls2.insert(Path::empty());
                    pls.insert(address, pls2);
                }
                identifier_invariants = pls;
            }

            for (address, identifier_paths) in identifier_invariants.into_iter() {
                if let Some(object_paths) = watch.objects_changed.get(&address) {
                    // address hasn't changed 
                    // but the object at that address has changed under some conditions 
                    for identifier_path in identifier_paths {
                        let mut invariants = possible_object_invariants(&identifier_path, object_paths);
                        if invariants.len() > 0 {
                            real_problem = true;
                        }
                        same.append(&mut invariants);
                    }
                } else {
                    // address hasn't changed 
                    // object at that address hasn't either
                    real_problem = true;
                    for identifier_path in identifier_paths {
                        same.insert(identifier_path);
                    }
                }
            }

            let mut new_problems = Vec::new();

            for problem in problems.iter() {
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
                source: vm.current_node().clone(),
                content: Box::new(content),
            };
            &CHANNEL.publish(message);
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
                    &AnalysisItem::Identifier (ref name) => vm.load_identifier(executors, name),
                    &AnalysisItem::Attribute (ref parent, ref name) => {
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
                        source: vm.current_node().clone(),
                        content: Box::new(content),
                    };
                    &CHANNEL.publish(message);
                }
            }
        }
    }
}

fn possible_identifier_invariants(old: &HashSet<Pointer>, changes: &Mapping) -> BTreeMap<Pointer, BTreeSet<Path>> {
    let mut all_reversals = BTreeMap::new();
    let mut all_changes = BTreeSet::new();

    for (path, _) in changes.iter() {
        all_changes.insert(path.clone());
    }

    for (path, address) in changes.iter(){
        let mut change_reversals = path.reverse();

        if old.contains(address) {
            change_reversals.push(path.clone());
        }

        for reversal in change_reversals.iter().rev() {
            for change in &all_changes {
                if reversal.contains(change) {
                    break;
                }
            }

            match all_reversals.entry(*address) {
                Entry::Vacant(v) => {
                    let mut acc = BTreeSet::new();
                    acc.insert(path.clone());
                    v.insert(acc);
                }
                Entry::Occupied(mut o) => {
                    o.get_mut().insert(path.clone());
                }
            };
        }
    }

    return all_reversals;
}

fn possible_object_invariants(parent_path: &Path, changes: &Vec<Path>) -> BTreeSet<Path> {
    // remove obsolete entries first, i.e.
    // (1, 5) and (1, 5, 9)
    let mut all_reversals = BTreeSet::new();

    for change in changes {
        let change_reversals = change.reverse();
        for reversal in change_reversals.iter().rev() {
            all_reversals.insert(reversal.clone());
        }
    }

    let mut possibilities = BTreeSet::new();
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
            possibilities.insert(new_path);
        }
    }

    return possibilities;
}