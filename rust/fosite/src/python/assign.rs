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
            //&NodeType::List { ref content } |
            //&NodeType::Sequence { ref content } => self.assign_to_iterable(content, mapping),
            &NodeType::Attribute { ref parent, ref attribute } => {
                self.assign_to_attribute(vm, executors, parent, attribute, mapping)
            }
            _ => panic!("unimplemented"),
        }
    }

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
