use core::*;

pub struct PythonUnOp { }

impl UnOpExecutor for PythonUnOp {
    fn execute(&self, env: Environment, value: &GastNode) -> ExecutionResult {
        let Environment { mut vm, executors } = env;

        let value_result = vm.execute(executors, value);
        let dependencies = value_result.dependencies;
        let changes = value_result.changes;

        let mut result_mapping = Mapping::new();

        for (path, address) in value_result.result {
            let t;
            {
                let o = vm.get_object(&address);
                t = *o.get_extension().last().unwrap();
            }
            
            let n = vm.object_of_type_pointer(&t);
            result_mapping.add_mapping(path, n);
        }

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        }
    }
}