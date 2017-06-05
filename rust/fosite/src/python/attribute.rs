use core::*;

use std::collections::BTreeSet;

pub struct PythonAttribute { }

impl AttributeExecutor for PythonAttribute {
    fn execute(&self, env: Environment, parent: &GastNode, name: &str) -> ExecutionResult {
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

        for &(ref parent_path, ref parent_address) in &parent_mapping {
            total_dependencies.push(AnalysisItem::Object(*parent_address));

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
            for (path, opt_address) in opt_mappings {
                if parent_path.mergeable(&path) {
                    let mut new_path = parent_path.clone();

                    // todo can probably remove this clone
                    new_path.merge_into(path);
                    actual_paths.push((new_path, opt_address));
                }
            }

            let num_paths = actual_paths.len();

            for (path, opt_address) in actual_paths {
                if let Some(address) = opt_address {
                    // make a method object is necessary
                    if vm.is_instance(&address, &"method".to_owned()) {
                        let method = vm.make_method_object(executors, parent_address, &address);
                        mapping.add_mapping(path, method);
                    } else {
                        mapping.add_mapping(path, address);
                    }
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
            if !unresolved.is_empty() {
                if !types.is_empty() {
                    for unmet in &unresolved {
                        // todo, add type information as well
                        error.insert(unmet.clone());
                    }
                    continue;
                }

                for tpe in &types {
                    for (path, opt_address) in
                        self.load_object_attribute(vm, executors, tpe, name) {
                        for original in &unresolved {
                            let mut new_path = path.clone();
                            for pls in original {
                                new_path.add_node(pls.clone());
                            }

                            if let Some(address) = opt_address {
                                // update watches in the VM 
                                vm.store_object_dependency(*parent_address);

                                // make a method object is necessary
                                if vm.is_instance(&address, &"function".to_owned()) {
                                    let method = vm.make_method_object(executors, parent_address, &address);
                                    mapping.add_mapping(new_path, method);
                                } else {
                                    mapping.add_mapping(new_path, address);
                                }
                            } else {
                                error.insert(new_path);
                            }
                        }
                    }
                }
            }
        }

        if !warning.is_empty() {
            let content = AttributeUnsafe::new(parent.to_string(), name.to_owned(), warning);
            let message = Message::Output {
                source: vm.current_node().clone(), 
                content: Box::new(content),
            };
            CHANNEL.publish(message);
        }

        if !error.is_empty() {
            let content = AttributeInvalid::new(parent.to_string(), name.to_owned(), error);
            let message = Message::Output { 
                source: vm.current_node().clone(), 
                content: Box::new(content),
            };
            CHANNEL.publish(message);
        }

        if let Some(item) = parent.kind.to_analysis_item() {
            total_dependencies.push(AnalysisItem::Attribute (
                Box::new(item.clone()),
                name.to_owned(),
            ));
            
            vm.store_identifier_dependency(AnalysisItem::Attribute (
                Box::new(item),
                name.to_owned(),
            ), &parent_mapping);
        }

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: total_changes,
            result: mapping,
        }
    }
}

impl PythonAttribute {
    fn load_object_attribute(&self,
                             vm: &VirtualMachine,
                             executors: &Executors,
                             address: &Pointer,
                             name: &str)
                             -> OptionalMapping {
        let mut unresolved = Vec::new();

        let object = vm.get_object(address);
        let opt_mappings = object.get_attribute(name);

        let mut result = OptionalMapping::new();

        for &(ref path, ref opt_address) in opt_mappings {
            if let Some(address) = *opt_address {
                result.add_mapping(path.clone(), Some(address));
            } else {
                unresolved.push(path.clone());
            }
        }

        if !unresolved.is_empty() {
            let types = object.get_extension();

            if !types.is_empty() {
                // can't go further up the hierarchy
                result.add_mapping(Path::empty(), None);
            }

            for tpe in types {
                let mut found = true;

                for (path, opt_address) in
                    self.load_object_attribute(vm, executors, tpe, name) {
                    if opt_address.is_none() {
                        found = false;
                    }

                    for original in &unresolved {
                        let mut new_path = path.clone();
                        for pls in original {
                            new_path.add_node(pls.clone());
                        }
                        result.add_mapping(new_path, opt_address);
                    }
                }

                if found {
                    // todo, technically we should adjust the unresolved vector now
                    // the next type only gets explored if this one returned nothing
                    break;
                }
            }
        }

        result
    }
}
