use core::*;

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

        total_dependencies.append(&mut dependencies);
        total_changes.append(&mut changes);

        let mut warning = BTreeSet::new();
        let mut error = BTreeSet::new();

        for (parent_path, parent_address) in parent_mapping.iter() {
            total_dependencies.push(AnalysisItem::Object(parent_address.clone()));

            let opt_mappings;
            let types;
            {
                let parent_object = vm.get_object(parent_address);
                opt_mappings = parent_object.get_attribute(name).clone();
                types = parent_object.get_extension().clone();
            }

            // copy the actual possible paths
            // need the amount of actual paths to decide whether or not to send a warning
            let mut actual_paths = Vec::new();
            for (path, opt_address) in opt_mappings.into_iter() {
                if parent_path.mergeable(&path) {
                    let mut new_path = parent_path.clone();

                    // todo can probably remove this clone
                    new_path.merge_into(path);
                    actual_paths.push((new_path, opt_address));
                }
            }

            let num_paths = actual_paths.len();

            for (path, opt_address) in actual_paths.into_iter() {
                if let Some(address) = opt_address {
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

                            if let Some(address) = opt_address {
                                // update watches in the VM 
                                vm.store_object_dependency(parent_address.clone());

                                mapping.add_mapping(new_path, address);
                            } else {
                                error.insert(new_path);
                            }
                        }
                    }
                }
            }
        }

        if warning.len() > 0 {
            let content = AttributeUnsafe::new(parent.to_string(), name.clone(), warning);
            let message = Message::Output {
                source: vm.current_node().clone(), 
                content: Box::new(content),
            };
            &CHANNEL.publish(message);
        }

        if error.len() > 0 {
            let content = AttributeInvalid::new(parent.to_string(), name.clone(), error);
            let message = Message::Output { 
                source: vm.current_node().clone(), 
                content: Box::new(content),
            };
            &CHANNEL.publish(message);
        }

        if let Some(item) = parent.kind.to_analysis_item() {
            total_dependencies.push(AnalysisItem::Attribute (
                Box::new(item.clone()),
                name.clone(),
            ));
            
            vm.store_identifier_dependency(AnalysisItem::Attribute (
                Box::new(item),
                name.clone(),
            ), &parent_mapping);
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
