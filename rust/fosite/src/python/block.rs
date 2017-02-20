use core::*;

pub struct PythonBlock { }

impl BlockExecutor for PythonBlock {
    fn execute(&self, env: Environment, content: &Vec<GastNode>) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let mut total_dependencies = Vec::new();
        let mut total_changes = Vec::new();

        let mut flow = FlowControl::Continue;

        for node in content {
            let intermediate = vm.execute(executors, node);

            let mut dependencies = intermediate.dependencies;
            let mut changes = intermediate.changes;

            total_dependencies.append(&mut dependencies);
            total_changes.append(&mut changes);

            match intermediate.flow {
                FlowControl::TerminateLoop | FlowControl::TerminateCall => {
                    flow = intermediate.flow;
                    break;
                },
                _ => ()
            }
        }

        let mut branch = None;
        
        {
            if let Some(node) = vm.current_branch() {
                branch = Some(node.clone());
            }
        }

        if branch.is_some() {
            vm.merge_until(&total_changes, branch);
        }

        return ExecutionResult {
            flow: flow,
            dependencies: total_dependencies,
            changes: total_changes,
            result: Mapping::new(),
        };
    }
}
