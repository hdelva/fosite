use core::*;

use std::collections::BTreeSet;

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
        let mut gen_result = vm.execute(executors, before);
        vm.toggle_watch();

        total_changes.append(&mut gen_result.changes);
        total_dependencies.append(&mut gen_result.dependencies);

        let mut result = self.branch(vm, executors, body, &gen_result.result);

        total_changes.append(&mut result.changes);
        total_dependencies.append(&mut result.dependencies);

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
              body: &GastNode,
              gen: &Mapping) -> ExecutionResult {
        //let restrictions = vm.get_loop_restrictions().clone();
                          
        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();

        let mut new_path = vm.current_path().clone();
        new_path.add_node(PathNode::Loop(vm.current_node().clone()));


        vm.push_path(new_path);

        // first iter
        let mut body_result = vm.execute(executors, body);

        total_changes.append(&mut body_result.changes);
        total_dependencies.append(&mut body_result.dependencies);

        vm.pop_path();

        self.check_changes(vm, gen);

        vm.merge_loop(&total_changes);

        return ExecutionResult {
            changes: total_changes,
            dependencies: total_dependencies,
            flow: FlowControl::Continue,
            result: Mapping::new(),
        };
    }

    fn check_changes(&self, vm: &mut VirtualMachine, gen: &Mapping) {
        let mut watch = vm.pop_watch();
        let mut actual = BTreeSet::new();

        for &(_, ref address) in gen.iter() {
            actual.insert(address.clone());
        }

        let mut problems = vec!();

        for (identifier, addresses) in watch.identifiers_before.into_iter() {
            for address in addresses.iter() {
                if !actual.contains(address){
                    continue;
                }

                if let Some(object_paths) = watch.objects_changed.get_mut(address) {
                    problems.append(object_paths);
                }
            }

            if let Some(mapping) = watch.identifiers_changed.get(&identifier) {
                for &(ref path, ref address) in mapping.iter() {
                    if !addresses.contains(&address){
                        continue;
                    }

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
}