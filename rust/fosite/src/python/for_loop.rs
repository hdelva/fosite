use core::*;

use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

pub struct PythonFor { }

impl ForEachExecutor for PythonFor {
    fn execute(&self,
               env: Environment,
               before: &GastNode,
               body: &GastNode)
               -> ExecutionResult {
        let Environment { vm, executors } = env;

        // register this node as a branch
        let id = vm.current_node().clone();
        vm.push_branch(id);

        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();

        vm.start_watch();
        let mut gen_result = vm.execute(executors, before);
        vm.toggle_watch();

        total_changes.append(&mut gen_result.changes);
        total_dependencies.append(&mut gen_result.dependencies);

        let mut result = self.branch(vm, executors, body);

        total_changes.append(&mut result.changes);
        total_dependencies.append(&mut result.dependencies);

        // register this node as a branch
        vm.pop_branch();

        return ExecutionResult {
            changes: total_changes,
            dependencies: total_dependencies,
            flow: FlowControl::Continue,
            result: Mapping::new(),
        };
    }
}

impl PythonFor {
    fn branch(&self,
              vm: &mut VirtualMachine,
              executors: &Executors,
              body: &GastNode) -> ExecutionResult {
                          
        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();

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
        let body_result = vm.execute(executors, body);
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
        let mut watch = vm.pop_watch();

        let mut problems = vec!();

        for (identifier, addresses) in watch.identifiers_before.into_iter() {
            for address in addresses {
                if let Some(object_paths) = watch.objects_changed.get_mut(&address) {
                    problems.append(object_paths);
                }
            }

            if let Some(mapping) = watch.identifiers_changed.get(&identifier) {
                for (path, _) in mapping.iter() {
                    problems.push(path.clone());
                }
            }
        }

        if problems.len() > 0 {
            let content = ForLoopChange::new(problems);
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