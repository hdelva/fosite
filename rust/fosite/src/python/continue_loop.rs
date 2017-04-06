use core::*;

pub struct PythonContinue { }

impl ContinueExecutor for PythonContinue {
    fn execute(&self, env: Environment) -> ExecutionResult {
        let Environment { vm, .. } = env;

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::TerminateLoop,
            dependencies: vec!(),
            changes: vec!(),
            result: result_mapping,
        };
    }
}