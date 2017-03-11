use core::*;

pub struct PythonList {}

impl ListExecutor for PythonList {
    fn execute(&self, env: Environment, content: &Vec<GastNode> ) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let type_name = "list".to_owned();
        let obj_ptr = vm.object_of_type(&type_name);

        let mut chunks = Vec::new();
        for node in content {
            let intermediate = vm.execute(executors, node);

            let mut chunk = CollectionChunk::empty();

            for (path, address) in intermediate.result.into_iter(){
                let kind = vm.get_object(&address).get_extension().first().unwrap();
                let repr = Representant::new(address, kind.clone(), Some(1), Some(1));
                chunk.add_representant(path, repr);    
            }
            
            chunks.push(chunk);
        }

        {
            let mut obj = vm.get_object_mut(&obj_ptr);
            obj.define_elements(chunks, Path::empty());
        }

        let mapping = Mapping::simple(Path::empty(), obj_ptr.clone());

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec!(),
            result: mapping,
        };

        return execution_result;
    }
}

pub struct PythonTuple {}

impl SequenceExecutor for PythonTuple {
    fn execute(&self, env: Environment, content: &Vec<GastNode> ) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let type_name = "tuple".to_owned();
        let obj_ptr = vm.object_of_type(&type_name);

        let mut chunks = Vec::new();
        for node in content {
            let intermediate = vm.execute(executors, node);

            let mut chunk = CollectionChunk::empty();

            for (path, address) in intermediate.result.into_iter(){
                let kind = vm.get_object(&address).get_extension().first().unwrap();
                let repr = Representant::new(address, kind.clone(), Some(1), Some(1));
                chunk.add_representant(path, repr);    
            }
            
            chunks.push(chunk);
        }

        {
            let mut obj = vm.get_object_mut(&obj_ptr);
            obj.define_elements(chunks, Path::empty());
        }

        let mapping = Mapping::simple(Path::empty(), obj_ptr.clone());

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec!(),
            result: mapping,
        };

        return execution_result;
    }
}

pub struct PythonSet {}

impl SetExecutor for PythonSet {
    fn execute(&self, env: Environment, content: &Vec<GastNode> ) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let type_name = "set".to_owned();
        let obj_ptr = vm.object_of_type(&type_name);

        let mut chunks = Vec::new();
        for node in content {
            let intermediate = vm.execute(executors, node);

            let mut chunk = CollectionChunk::empty();

            for (path, address) in intermediate.result.into_iter(){
                let kind = vm.get_object(&address).get_extension().first().unwrap();
                let repr = Representant::new(address, kind.clone(), Some(1), Some(1));
                chunk.add_representant(path, repr);    
            }
            
            chunks.push(chunk);
        }

        {
            let mut obj = vm.get_object_mut(&obj_ptr);
            obj.define_elements(chunks, Path::empty());
        }

        let mapping = Mapping::simple(Path::empty(), obj_ptr.clone());

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec!(),
            result: mapping,
        };

        return execution_result;
    }
}

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
        let type_name = "string".to_owned();

        let string_type = vm.knowledge().get_type(&type_name).unwrap().clone();

        let string_ptr = vm.object_of_type(&type_name);
        let character_ptr = vm.object_of_type(&type_name);

        {
            let mut char_object = vm.get_object_mut(&character_ptr);
            let repr = Representant::new(character_ptr, string_type, Some(1), Some(1));
            let mut chunk = CollectionChunk::empty();
            chunk.add_representant(Path::empty(), repr);
            char_object.define_elements(vec!(chunk), Path::empty());
        }

        {
            let mut string_object = vm.get_object_mut(&string_ptr);
            let repr = Representant::new(character_ptr, string_type, None, None);
            let mut chunk = CollectionChunk::empty();
            chunk.add_representant(Path::empty(), repr);
            string_object.define_elements(vec!(chunk), Path::empty());
        }

        let mapping = Mapping::simple(Path::empty(), string_ptr.clone());

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec!(),
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
