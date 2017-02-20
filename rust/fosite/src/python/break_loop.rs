use core::*;

pub struct PythonBreak { }

impl BreakExecutor for PythonBreak {
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