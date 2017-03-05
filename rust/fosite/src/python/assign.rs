use core::*;

pub struct PythonAssign { }

impl AssignExecutor for PythonAssign {
    fn execute(&self,
               env: Environment,
               targets: &Vec<GastNode>,
               value: &GastNode)
               -> ExecutionResult {
        let Environment { vm, executors } = env;

        let value_execution = vm.execute(executors, value);

        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();

        let mut value_changes = value_execution.changes;
        let mut value_dependencies = value_execution.dependencies;
        let value_mapping = value_execution.result;

        total_changes.append(&mut value_changes);
        total_dependencies.append(&mut value_dependencies);

        for target in targets {
            let target_result = self.assign_to_target(vm, executors, target, &value_mapping);
            let mut target_dependencies = target_result.dependencies;
            let mut target_changes = target_result.changes;

            total_changes.append(&mut target_changes);
            total_dependencies.append(&mut target_dependencies);
        }

        let mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: total_changes,
            result: mapping,
        };
    }
}

impl PythonAssign {
    fn assign_to_target(&self,
                        vm: &mut VirtualMachine,
                        executors: &Executors,
                        target: &GastNode,
                        mapping: &Mapping)
                        -> ExecutionResult {
        match &target.kind {
            &NodeType::Identifier { ref name } => {
                self.assign_to_identifier(vm, executors, name, mapping)
            }
            &NodeType::List { ref content } |
            &NodeType::Sequence { ref content } => {
                self.assign_to_iterable(vm, executors, content, mapping)
            },
            &NodeType::Attribute { ref parent, ref attribute } => {
                self.assign_to_attribute(vm, executors, parent, attribute, mapping)
            }
            _ => panic!("unimplemented"),
        }
    }

    fn _assign_to_iterable<F>(&self, 
                           vm: &mut VirtualMachine,
                           executors: &Executors,
                           content: &[GastNode], 
                           mapping: &Mapping,
                           num: usize,
                           fun: F)
                           -> ExecutionResult
                           where F: Fn(&Object) -> Vec<Mapping> {
        let mut dependencies = Vec::new();
        let mut changes = Vec::new();

        let mut value_mappings = Vec::new();

        for (path, address) in mapping.iter() {
            let object = vm.get_object(address);

            for (_, min, max) in object.size_range() {
                if let Some(max) = max {
                    if max < num {
                        //todo warning
                        //blacklist this path
                    }
                } 

                if let Some(min) = min {
                    if min > num {
                        //todo warning
                        //blacklist this path
                    }
                }
            }

            // needs current node information to form the path
            let possibilities = fun(object);

            // iterate over all possible elements
            // combine the elements's path with the object's path 
            // store the results in value_mappings 
            //
            // maintain the same order
            // the first element of value_mappings contains 
            // the the mapping for the first assign target
            for (index, mapping) in possibilities.into_iter().enumerate() {
                if value_mappings.len() <= index {
                    value_mappings.push(Mapping::new());
                }

                let ref mut new_mapping = value_mappings[index];

                for (path, address) in mapping.into_iter() {
                    let mut new_path = path.clone();
                    for node in path.into_iter() {
                        new_path.add_node(node);
                    }
                    new_mapping.add_mapping(new_path, address);
                }
            }
        }

        for (target, target_mapping) in content.iter().zip(value_mappings) {
            let mut partial_result = self.assign_to_target(vm, executors, target, &target_mapping);
            changes.append(&mut partial_result.changes);
            dependencies.append(&mut partial_result.dependencies);
        }

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        };
    }

    fn slice(&self, 
              vm: &mut VirtualMachine,
              executors: &Executors,
              content: &[GastNode], 
              mapping: &Mapping,
              left: i16,
              right: i16)
              -> Mapping {
        let mut result_mapping = Mapping::new();

        for (path, address) in mapping.iter() {
            let elements;
            {
                let object = vm.get_object(address);
                // one way to fix an off-by-one error
                elements = object.slice_elements(left, right);
            }

            let type_name = "list".to_owned();
            let slice_ptr = vm.object_of_type(&type_name);
            {
                let mut object = vm.get_object_mut(&slice_ptr);
                object.set_elements(elements);
            }

            result_mapping.add_mapping(path.clone(), slice_ptr);
        }

        return result_mapping;
    }

    fn assign_to_iterable(&self, 
                          vm: &mut VirtualMachine,
                          executors: &Executors,
                          content: &[GastNode], 
                          mapping: &Mapping) 
                          -> ExecutionResult {
        let mut dependencies = Vec::new();
        let mut changes = Vec::new();

        let current_node = vm.current_node().clone();

        let num = content.len();

        let mut split = None;
        for (index, target) in content.iter().enumerate() {
            let &GastNode {ref kind, ..} = target;
            if let &NodeType::UnOp { ref value, .. } = kind {
                split = Some((index, value));
            }
        }

        if let Some((index, target)) = split {
            let fun = |obj: &Object| {
                obj.get_first_n_elements(index as i16, &current_node)
            };

            let mut partial_result = self._assign_to_iterable(vm, executors, &content[..index], mapping, index, fun);
            changes.append(&mut partial_result.changes);
            dependencies.append(&mut partial_result.dependencies);

            let pls = num - index - 1;

            let fun = |obj: &Object| {
                obj.get_last_n_elements(pls as i16, &current_node)
            };

            let mut partial_result = self._assign_to_iterable(vm, executors, &content[index + 1..], mapping, pls, fun);
            changes.append(&mut partial_result.changes);
            dependencies.append(&mut partial_result.dependencies);

            let slice_mapping = self.slice(vm, executors, content, mapping, index as i16, pls as i16);
            let mut partial_result = self.assign_to_target(vm, executors, target, &slice_mapping);
        } else {
            let fun = |obj: &Object| {
                obj.get_first_n_elements(num as i16, &current_node)
            };

            let mut partial_result = self._assign_to_iterable(vm, executors, &content[..num], mapping, num, fun);
            changes.append(&mut partial_result.changes);
            dependencies.append(&mut partial_result.dependencies);
        }

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        };
    }

/*
    fn assign_to_iterable(&self, 
                          vm: &mut VirtualMachine,
                          executors: &Executors,
                          content: &[GastNode], 
                          mapping: &Mapping) 
                          -> ExecutionResult {
        let mut dependencies = Vec::new();
        let mut changes = Vec::new();

        let current_node = vm.current_node().clone();

        let num = content.len();

        let mut value_mappings = Vec::new();

        let mut split = None;
        for (index, target) in content.iter().enumerate() {
            let &GastNode {ref kind, ..} = target;
            if let &NodeType::UnOp { ref value, .. } = kind {
                split = Some((index, value));
            }
        }


        let mut fun = |obj: Object, num, node| {
            obj.get_first_n_elements(num, node)
        };


        for (path, address) in mapping.iter() {
            let object = vm.get_object(address);

            for (_, min, max) in object.size_range() {
                if let Some(max) = max {
                    if max < num {
                        //todo warning
                        //blacklist this path
                    }
                } 

                if let Some(min) = min {
                    if min > num {
                        //todo warning
                        //blacklist this path
                    }
                }
            }

            // needs current node information to form the path
            let possibilities = object.get_first_n_elements(num as i16, &current_node);

            // iterate over all possible elements
            // combine the elements's path with the object's path 
            // store the results in value_mappings 
            //
            // maintain the same order
            // the first element of value_mappings contains 
            // the the mapping for the first assign target
            for (index, mapping) in possibilities.into_iter().enumerate() {
                if value_mappings.len() <= index {
                    value_mappings.push(Mapping::new());
                }

                let ref mut new_mapping = value_mappings[index];

                for (path, address) in mapping.into_iter() {
                    let mut new_path = path.clone();
                    for node in path.into_iter() {
                        new_path.add_node(node);
                    }
                    new_mapping.add_mapping(new_path, address);
                }
            }
        }

        for (target, target_mapping) in content.iter().zip(value_mappings) {
            let mut partial_result = self.assign_to_target(vm, executors, target, &target_mapping);
            changes.append(&mut partial_result.changes);
            dependencies.append(&mut partial_result.dependencies);
        }

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        };
    }
*/
    fn assign_to_attribute(&self,
                           vm: &mut VirtualMachine,
                           executors: &Executors,
                           parent: &GastNode,
                           attribute: &String,
                           mapping: &Mapping)
                           -> ExecutionResult {

        // todo get rid of clone
        let mapping = mapping.clone().augment(PathNode::Assignment(vm.current_node(),
                                                                   format!("{}.{}",
                                                                           parent.to_string(),
                                                                           attribute)));

        let parent_result = vm.execute(executors, parent);

        let result = parent_result.result;
        let dependencies = parent_result.dependencies;

        let parent_mapping = result;
        let mut changes = Vec::new();

        // add the attribute identifier changes
        for dependency in dependencies.iter() {
            if !dependency.is_object() {
                changes.push(AnalysisItem::Attribute {
                    parent: Box::new(dependency.clone()),
                    name: attribute.clone(),
                });
            }
        }

        // add the object changes
        // perform the assignment
        for (_, parent_address) in parent_mapping.iter() {
            // todo this clone shouldn't be necessary
            let current_path = vm.current_path().clone();

            changes.push(AnalysisItem::Object { address: parent_address.clone(), path: Some(current_path.clone()) });

            let mut parent_object = vm.get_object_mut(parent_address);
            parent_object.assign_attribute(attribute.clone(), current_path, mapping.clone())
        }

        // notify the vm a mapping has changed
        // used to update watches
        // dirty fix, assumes the first change is the relevant one
        vm.notify_change(changes.first().unwrap().clone(), mapping.clone());

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        };
    }

    fn assign_to_identifier(&self,
                            vm: &mut VirtualMachine,
                            _: &Executors,
                            target: &String,
                            mapping: &Mapping)
                            -> ExecutionResult {
        let changes = vec![AnalysisItem::Identifier { name: target.clone() }];

        // todo get rid of clone
        let mapping = mapping.clone()
            .augment(PathNode::Assignment(vm.current_node(), target.clone()));

        let path = vm.current_path().clone();

        {
            let mut scope = vm.last_scope_mut();
            scope.set_mapping(target.clone(), path, mapping.clone());
        }

        // notify the vm a mapping has changed
        // used to update watches
        // dirty fix, assumes the first change is the relevant one
        vm.notify_change(changes.first().unwrap().clone(), mapping.clone());

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: changes,
            result: result_mapping,
        };
    }
}
