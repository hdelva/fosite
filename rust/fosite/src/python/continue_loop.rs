use core::*;

pub struct PythonContinue { }

impl ContinueExecutor for PythonContinue {
    fn execute(&self, env: Environment) -> ExecutionResult {
        let Environment { vm, .. } = env;

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));
        
        let relevant_node = vm.current_path().iter().last().unwrap().clone();
        let mut relevant_path = Path::empty();
        relevant_path.add_node(relevant_node);

        return ExecutionResult {
            flow: FlowControl::TerminateLoop,
            dependencies: vec!(),
            changes: vec!(),
            result: result_mapping,
        };
    }
}