use core::*;

use std::collections::HashMap;
use std::collections::BTreeSet;

pub struct PythonAttribute { }

impl AttributeExecutor for PythonAttribute {
    fn execute(&self, env: Environment, parent: &GastNode, name: &String) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let parent_result = vm.execute(executors, parent);

        let mut total_dependencies = Vec::new();
        let mut total_changes = Vec::new();
        let mut mapping = Mapping::new();

        // which assumptions still need a valid mapping
        let mut unresolved = BTreeSet::new();

        let parent_mapping = parent_result.result;
        let mut dependencies = parent_result.dependencies;
        let mut changes = parent_result.changes;

        for dependency in dependencies.iter() {
            total_dependencies.push(AnalysisItem::Attribute {
                parent: Box::new(dependency.clone()),
                name: name.clone(),
            });
        }

        total_dependencies.append(&mut dependencies);
        total_changes.append(&mut changes);

        let mut warning = BTreeSet::new();
        let mut error = BTreeSet::new();

        for (parent_path, parent_address) in parent_mapping.iter() {
            dependencies.push(AnalysisItem::Object { address: parent_address.clone() });

            let parent_object = vm.get_object(parent_address);
            let opt_mappings = parent_object.get_attribute(name);

            // copy the actual possible paths
            // need the amount of actual paths to decide whether or not to send a warning
            let mut actual_paths = Vec::new();
            for (path, opt_address) in opt_mappings.iter() {
                if parent_path.mergeable(&path) {
                    let mut new_path = parent_path.clone();

                    // todo can probably remove this clone
                    new_path.merge_into(path.clone());
                    actual_paths.push((new_path, opt_address));
                }
            }

            let num_paths = actual_paths.len();

            for (path, opt_address) in actual_paths.into_iter() {
                if let &Some(address) = opt_address {
                    mapping.add_mapping(path, address.clone());
                } else {
                    unresolved.insert(path.clone());

                    if num_paths > 1 {
                        // having a single None is fine
                        // probably a class method then
                        warning.insert(path);
                    }
                }
            }

            // look for the attribute in its types
            if unresolved.len() > 0 {
                let types = parent_object.get_extension();

                if types.len() == 0 {
                    for unmet in unresolved.iter() {
                        // todo, add type information as well
                        error.insert(unmet.clone());
                    }
                    continue;
                }

                for tpe in types.iter() {
                    for (path, opt_address) in
                        self.load_object_attribute(vm, executors, tpe, name).into_iter() {
                        for original in unresolved.iter() {
                            let mut new_path = path.clone();
                            for pls in original.iter() {
                                new_path.add_node(pls.clone());
                            }

                            if opt_address.is_none() {
                                // todo, add type information as well
                                error.insert(new_path);
                                continue;
                            } else {
                                mapping.add_mapping(new_path, opt_address.unwrap());
                            }
                        }
                    }
                }
            }
        }

        if warning.len() > 0 {
            let mut items = HashMap::new();

            items.insert("parent".to_owned(), MessageItem::String(parent.to_string()));
            items.insert("name".to_owned(), MessageItem::String(name.clone()));

            let mut path_count = 0;
            for path in warning {
                if error.contains(&path) {
                    continue;
                }
                items.insert(format!("path {}", path_count),
                             MessageItem::Path(path.clone()));
                path_count += 1;
            }

            if path_count > 0 {
                let message = Message::Warning {
                    source: vm.current_node(),
                    kind: WATTRIBUTE_UNSAFE,
                    content: items,
                };
                &CHANNEL.publish(message);
            }
        }

        if error.len() > 0 {
            let mut items = HashMap::new();

            items.insert("parent".to_owned(), MessageItem::String(parent.to_string()));
            items.insert("name".to_owned(), MessageItem::String(name.clone()));

            let mut path_count = 0;
            for path in error {
                items.insert(format!("path {}", path_count),
                             MessageItem::Path(path.clone()));
                path_count += 1;
            }

            let message = Message::Error {
                source: vm.current_node(),
                kind: EATTRIBUTE_INVALID,
                content: items,
            };
            &CHANNEL.publish(message);
        }

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: total_changes,
            result: mapping,
        };
    }
}

impl PythonAttribute {
    fn load_object_attribute(&self,
                             vm: &VirtualMachine,
                             executors: &Executors,
                             address: &Pointer,
                             name: &String)
                             -> OptionalMapping {
        let mut unresolved = Vec::new();

        let object = vm.get_object(address);
        let opt_mappings = object.get_attribute(name);

        let mut result = OptionalMapping::new();

        for (path, opt_address) in opt_mappings.iter() {
            if let &Some(address) = opt_address {
                result.add_mapping(path.clone(), Some(address.clone()));
            } else {
                unresolved.push(path.clone());
            }
        }

        if unresolved.len() > 0 {
            let types = object.get_extension();

            if types.len() == 0 {
                // can't go further up the hierarchy
                result.add_mapping(Path::empty(), None);
            }

            for tpe in types {
                let mut found = true;

                for (path, opt_address) in
                    self.load_object_attribute(vm, executors, tpe, name).into_iter() {
                    if opt_address.is_none() {
                        found = false;
                    }

                    for original in unresolved.iter() {
                        let mut new_path = path.clone();
                        for pls in original.iter() {
                            new_path.add_node(pls.clone());
                        }
                        result.add_mapping(new_path, opt_address.clone());
                    }
                }

                if found {
                    // todo, technically we should adjust the unresolved vector now
                    // the next type only gets explored if this one returned nothing
                    break;
                }
            }
        }

        return result;
    }
}