use core::*;

pub struct PythonNegate { }

impl NegateExecutor for PythonNegate {
    fn execute(&self, env: Environment, value: &GastNode) -> ExecutionResult {
        let Environment { mut vm, executors } = env;

        let value_result = vm.execute(executors, value);
        let dependencies = value_result.dependencies;
        let changes = value_result.changes;

        let ptr = vm.object_of_type(&"bool".to_owned());

        let result_mapping = Mapping::simple(Path::empty(), ptr);

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        }
    }
}