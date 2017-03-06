use core::*;

use std::collections::HashMap;
use std::collections::BTreeSet;
use std::collections::hash_map::Entry;

pub struct PythonBoolOp { }

impl BoolOpExecutor for PythonBoolOp {
    fn execute(&self,
               env: Environment,
               left: &GastNode,
               op: &String,
               right: &GastNode)
               -> ExecutionResult {
        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();
        let mut result = Mapping::new();

        let Environment { vm, executors } = env;

        let mut left_result = vm.execute(executors, left);
        let left_mapping = left_result.result;
        total_changes.append(&mut left_result.changes);
        total_dependencies.append(&mut left_result.dependencies);

        let mut right_result = vm.execute(executors, right);
        let right_mapping = right_result.result;
        total_changes.append(&mut right_result.changes);
        total_dependencies.append(&mut right_result.dependencies);

        let mut error = HashMap::new();
        let mut warning = HashMap::new();

        let t = vm.knowledge().constant(&"True".to_owned());
        let f = vm.knowledge().constant(&"False".to_owned());
        
        let op: &str = &*op; 

        for (left_path, left_address) in left_mapping.iter() {
            for (right_path, right_address) in right_mapping.iter() {
                if !left_path.mergeable(right_path) {
                    continue;
                }

                let mut new_path = left_path.clone();

                // todo can probably avoid this clone
                new_path.merge_into(right_path.clone());

                let mut type_name = "None".to_owned();

                // the in operator isn't reflexive
                // bit of an annoying special case
                if op == "in" || op == "not in" {
                    let object = vm.get_object(right_address);

                    let ancestors = vm.ancestors(right_address);

                    for ancestor in ancestors.iter().rev() {
                        type_name = vm.knowledge().get_type_name(ancestor).clone();
                        if vm.knowledge().operation_supported(&type_name, &op.to_owned()) {
                            break;
                        }
                    }
                } else {
                    // todo, bit of a hack
                    // concludes that if the most recently defined type supports addition
                    // that the entire thing does
                    // reality is more complicated, and full of runtime type checks
                    let ancestors = vm.common_ancestor(left_address, right_address);

                    for ancestor in ancestors.iter().rev() {
                        type_name = vm.knowledge().get_type_name(ancestor).clone();
                        if vm.knowledge().operation_supported(&type_name, &op.to_owned()) {
                            break;
                        }
                    }

                    // object is a common ancestor of all actual objects
                    // if this is the only common ancestor, the two objects can't be compared 
                    if ancestors.len() == 1  {
                        let left_object = vm.get_object(left_address);
                        let left_type = left_object.get_extension().first().unwrap();
                        let left_type_name = vm.knowledge().get_type_name(left_type).clone();
                        let right_object = vm.get_object(right_address);
                        let right_type = right_object.get_extension().first().unwrap();
                        let right_type_name = vm.knowledge().get_type_name(right_type).clone();

                        // special case 
                        // don't throw warning when people are checking for None
                        if left_type_name  == "NoneType".to_owned() || 
                           right_type_name == "NoneType".to_owned() {
                               continue;
                        }

                        match warning.entry((left_type_name, right_type_name)) {
                            Entry::Vacant(o) => {
                                let mut left_set = BTreeSet::new();
                                let mut right_set = BTreeSet::new();
                                left_set.insert(left_path.clone());
                                right_set.insert(right_path.clone());
                                o.insert((left_set, right_set));
                            }
                            Entry::Occupied(mut entry) => {
                                let &mut (ref mut left_set, ref mut right_set) = entry.get_mut();
                                left_set.insert(left_path.clone());
                                right_set.insert(right_path.clone());
                            }
                        }
                    } 
                }

                if vm.knowledge().operation_supported(&type_name, &op.to_owned()) {
                    let new_object = match op {
                        "==" | "is" | ">=" | "<=" => {
                            if *left_address == *right_address {
                                t
                            } else {
                                vm.object_of_type(&"bool".to_owned())
                            }
                        },
                        "!=" | "is not" => {
                            if *left_address == *right_address {
                                f
                            } else {
                                vm.object_of_type(&"bool".to_owned())
                            }
                        },
                        _ => vm.object_of_type(&"bool".to_owned()),
                    };

                    result.add_mapping(new_path, new_object);
                } else {
                    let left_object = vm.get_object(left_address);
                    let left_type = left_object.get_extension().first().unwrap();
                    let left_type_name = vm.knowledge().get_type_name(left_type).clone();
                    let right_object = vm.get_object(right_address);
                    let right_type = right_object.get_extension().first().unwrap();
                    let right_type_name = vm.knowledge().get_type_name(right_type).clone();

                    match error.entry((left_type_name, right_type_name)) {
                        Entry::Vacant(o) => {
                            let mut left_set = BTreeSet::new();
                            let mut right_set = BTreeSet::new();
                            left_set.insert(left_path.clone());
                            right_set.insert(right_path.clone());
                            o.insert((left_set, right_set));
                        }
                        Entry::Occupied(mut entry) => {
                            let &mut (ref mut left_set, ref mut right_set) = entry.get_mut();
                            left_set.insert(left_path.clone());
                            right_set.insert(right_path.clone());
                        }
                    }
                }
            }
        }

        if warning.len() > 0 {
            let mut items = HashMap::new();

            let mut comb_count = 0;
            for (types, paths) in warning {
                let (left_type, right_type) = types;

                items.insert(format!("combination {} left", comb_count),
                             MessageItem::String(left_type.to_owned()));
                items.insert(format!("combination {} right", comb_count),
                             MessageItem::String(right_type.to_owned()));

                let (left_paths, right_paths) = paths;

                let mut path_count = 0;
                for path in left_paths.into_iter() {
                    items.insert(format!("combination {} left {}", comb_count, path_count),
                                 MessageItem::Path(path));
                    path_count += 1;
                }

                let mut path_count = 0;
                for path in right_paths.into_iter() {
                    items.insert(format!("combination {} right {}", comb_count, path_count),
                                 MessageItem::Path(path));
                    path_count += 1;
                }

                comb_count += 1;
            }

            let message = Message::Warning {
                source: vm.current_node(),
                kind: WBOOLOP,
                content: items,
            };
            &CHANNEL.publish(message);
        }

        if error.len() > 0 {
            let mut items = HashMap::new();

            items.insert("operation".to_owned(), MessageItem::String(op.to_owned()));

            let mut comb_count = 0;
            for (types, paths) in error {
                let (left_type, right_type) = types;

                items.insert(format!("combination {} left", comb_count),
                             MessageItem::String(left_type.to_owned()));
                items.insert(format!("combination {} right", comb_count),
                             MessageItem::String(right_type.to_owned()));

                let (left_paths, right_paths) = paths;

                let mut path_count = 0;
                for path in left_paths.into_iter() {
                    items.insert(format!("combination {} left {}", comb_count, path_count),
                                 MessageItem::Path(path));
                    path_count += 1;
                }

                let mut path_count = 0;
                for path in right_paths.into_iter() {
                    items.insert(format!("combination {} right {}", comb_count, path_count),
                                 MessageItem::Path(path));
                    path_count += 1;
                }

                comb_count += 1;
            }

            let message = Message::Error {
                source: vm.current_node(),
                kind: EBOOLOP,
                content: items,
            };
            &CHANNEL.publish(message);
        }

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: total_changes,
            result: result,
        };

        return execution_result;
    }
}
