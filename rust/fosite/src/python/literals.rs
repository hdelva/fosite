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
        let Environment { mut vm, .. } = env;
        let path = vm.current_path().clone();
        let type_name = "string".to_owned();

        let string_type = vm.knowledge().get_type(&type_name).unwrap().clone();
        let string_ptr = vm.object_of_type(&type_name);
        let character_ptr = vm.object_of_type(&type_name);

        {
            let mut char_object = vm.get_object_mut(&character_ptr);
            let repr = Representant::new(character_ptr, string_type);
            char_object.define_elements(vec!(CollectionChunk::new(None, None, repr)), path.clone());
        }

        {
            let mut string_object = vm.get_object_mut(&string_ptr);
            let repr = Representant::new(character_ptr, string_type);
            string_object.define_elements(vec!(CollectionChunk::new(Some(1), Some(1), repr)), path.clone());
        }

        let mapping = Mapping::simple(Path::empty(), string_ptr.clone());

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec!(AnalysisItem::Object {address: string_ptr, path: Some(path)}),
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
