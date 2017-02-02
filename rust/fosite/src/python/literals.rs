use core::*;

pub struct PythonBoolean { }

impl BooleanExecutor for PythonBoolean {
    fn execute(&self, env: Environment, value: bool) -> ExecutionResult {
        let Environment { vm, executors } = env;

        if value {
            vm.load_identifier(executors, &"True".to_owned())
        } else {
            vm.load_identifier(executors, &"False".to_owned())
        }
    }
}

pub struct PythonString { }

impl StringExecutor for PythonString {
    fn execute(&self, env: Environment) -> ExecutionResult {
        let Environment { vm, .. } = env;
        let type_name = "string".to_owned();
        let pointer = vm.object_of_type(&type_name);

        let mapping = Mapping::simple(Path::empty(), pointer.clone());

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![],
            result: mapping,
        };

        return execution_result;
    }
}

pub struct PythonInt { }

impl IntExecutor for PythonInt {
    fn execute(&self, env: Environment) -> ExecutionResult {
        let Environment { vm, .. } = env;
        let type_name = "int".to_owned();
        let pointer = vm.object_of_type(&type_name);

        let mapping = Mapping::simple(Path::empty(), pointer.clone());

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![],
            result: mapping,
        };

        return execution_result;
    }
}

pub struct PythonFloat { }

impl FloatExecutor for PythonFloat {
    fn execute(&self, env: Environment) -> ExecutionResult {
        let Environment { vm, .. } = env;
        let type_name = "float".to_owned();
        let pointer = vm.object_of_type(&type_name);

        let mapping = Mapping::simple(Path::empty(), pointer.clone());

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![],
            result: mapping,
        };

        return execution_result;
    }
}
