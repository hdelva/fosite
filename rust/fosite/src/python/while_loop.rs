use core::*;

use core::Path;
use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use std::iter::FromIterator;

pub struct PythonWhile { }

impl WhileExecutor for PythonWhile {
    fn execute(&self,
               env: Environment,
               test: &GastNode,
               body: &GastNode)
               -> ExecutionResult {
        let Environment { vm, executors } = env;

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

        let total = test_result.result.len();

        // split up the test result into yes/no/maybe
        for (path, address) in test_result.result.into_iter() {
            if address == f {
                no.push(path);
            }
        }

        self.branch(vm, executors, body, no, total_changes, total_dependencies)
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
                          
        let mut total_changes = HashSet::from_iter(c.into_iter());
        // rust can't infer the type?
        let mut total_dependencies: HashSet<_> = HashSet::from_iter(d.into_iter());

        let last_path = vm.pop_path();

        let mut positive = last_path.clone();
        positive.add_node(PathNode::Loop(vm.current_node(), true));
        let mut negative = last_path.clone();
        negative.add_node(PathNode::Loop(vm.current_node(), false));

        vm.push_path(positive);
        vm.add_restrictions(no);
        let body_result = vm.execute(executors, body);
        vm.drop_restrictions();
        let _ = vm.pop_path();

        let mut identifier_changed = false;

        let changes = body_result.changes;
        let dependencies = body_result.dependencies;

        for change in &changes {
            total_changes.insert(change.clone());

            if let &AnalysisItem::Identifier { .. } = change {
                identifier_changed = true;
            }
        }

        for dependency in &dependencies {
            total_dependencies.insert(dependency.clone());
        }

        vm.change_branch(identifier_changed, &total_changes);

        // restore the old path
        vm.push_path(last_path);

        self.check_changes(vm, &total_changes);

        // labels all changes made with the Loop id
        vm.merge_branches(&total_changes);

        self.check_types(vm, executors, &total_changes);

        // todo any way around the cloning?
        return ExecutionResult {
            changes: Vec::from_iter(total_changes.into_iter()),
            dependencies: Vec::from_iter(total_dependencies.into_iter()),
            flow: FlowControl::Continue,
            result: Mapping::new(),
        };
    }

    fn check_changes(&self, vm: &mut VirtualMachine, changes: &HashSet<AnalysisItem>) {
        let watch = vm.pop_watch();
        let Watch {before, after, ..} = watch;

        let mut relevant_objects = BTreeSet::new();
        let mut changed_objects = BTreeSet::new();

        for (_, mapping) in before.iter() {
            for (path, address) in mapping.iter() {
                relevant_objects.insert(address.clone());
            } 
        }

        for change in changes {
            if let &AnalysisItem::Object {address, ref path} = change {
                if relevant_objects.contains(&address) {
                    //todo, changed_objects should be Pointer->Vec<Path>
                    changed_objects.insert(address.clone());
                }
            }
        }

        if before.len() == 0 {
            //todo warning, trivial loop condition
        } else {
            let (identifier, mapping)  = before.iter().next().unwrap();

            let new_mapping = after.get(&identifier).unwrap();

            let mut problems = vec!(Path::empty());

            for (_, address) in mapping.iter() {
                // me too thanks
                let mut same = Vec::new();

                for (new_path, new_address) in new_mapping.iter() {
                    if address == new_address && !changed_objects.contains(address) {
                        same.push(new_path.clone());
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

            let mut items = HashMap::new();
            let mut count = 0;
            for problem in problems.into_iter() {
                items.insert(format!("path {}", count), MessageItem::Path(problem));
                count += 1;
            }

            let message = Message::Warning {
                source: vm.current_node(),
                kind: WWHILE_LOOP,
                content: items,
            };

            &CHANNEL.publish(message);
        }
    }

    fn check_types(&self,
             vm: &mut VirtualMachine,
             executors: &Executors,
             changes: &HashSet<AnalysisItem>) {
        for change in changes {
            if !change.is_object() {
                let mut all_types = HashMap::new();

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
                    let tpe = object.get_extension()[0];

                    match all_types.entry(tpe.clone()) {
                        Entry::Vacant(v) => {
                            v.insert(vec![path.clone()]);
                        }
                        Entry::Occupied(mut o) => {
                            o.get_mut().push(path.clone());
                        }
                    };
                }

                if all_types.len() > 1 {
                    let mut items = HashMap::new();

                    items.insert("name".to_owned(), MessageItem::String(change.to_string()));

                    let mut type_count = 0;
                    for (tpe, paths) in all_types {
                        let type_name = vm.knowledge().get_type_name(&tpe);
                        items.insert(format!("type {}", type_count),
                                     MessageItem::String(type_name.to_owned()));

                        let mut path_count = 0;
                        for path in paths {
                            items.insert(format!("type {} path {}", type_count, path_count),
                                         MessageItem::Path(path.clone()));
                            path_count += 1;
                        }
                        type_count += 1;
                    }

                    let kind = if change.is_identifier() {
                        WIDENTIFIER_POLY_TYPE
                    } else {
                        WATTRIBUTE_POLY_TYPE
                    };

                    let message = Message::Warning {
                        source: vm.current_node(),
                        kind: kind,
                        content: items,
                    };

                    &CHANNEL.publish(message);
                }
            }
        }
    }
}
