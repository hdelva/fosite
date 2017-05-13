use core::*;

pub struct PythonReturn {

}

impl ReturnExecutor for PythonReturn {
    fn execute(&self, env: Environment, value: &GastNode) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let mut changes = Vec::new();
        let mut dependencies = Vec::new();

        let mut aresult = vm.execute(executors, value);
        changes.append(&mut aresult.changes);
        dependencies.append(&mut aresult.dependencies);

        let path = vm.current_path().clone();
        vm.add_result(path, aresult.result);

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::TerminateCall,
            dependencies: vec!(),
            changes: vec!(),
            result: result_mapping,
        };
    }
}