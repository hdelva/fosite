use core::*;

pub struct PythonList {}

impl ListExecutor for PythonList {
    fn execute(&self, env: Environment, content: &[GastNode] ) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let type_name = "list".to_owned();
        let obj_ptr = vm.object_of_type(&type_name);

        make_collection(vm, executors, obj_ptr, content)
    }
}

pub struct PythonTuple {}

impl SequenceExecutor for PythonTuple {
    fn execute(&self, env: Environment, content: &[GastNode] ) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let type_name = "tuple".to_owned();
        let obj_ptr = vm.object_of_type(&type_name);

        make_collection(vm, executors, obj_ptr, content)
    }
}

pub struct PythonSet {}

impl SetExecutor for PythonSet {
    fn execute(&self, env: Environment, content: &[GastNode] ) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let type_name = "set".to_owned();
        let obj_ptr = vm.object_of_type(&type_name);

        make_collection(vm, executors, obj_ptr, content)
    }
}

pub struct PythonDict {}

impl DictExecutor for PythonDict {
    fn execute(&self, env: Environment, content: &[GastNode] ) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let dict_type = "dict".to_owned();
        let dict_ptr = vm.object_of_type(&dict_type);

        let set_type = "set".to_owned();
        let keys_ptr = vm.object_of_type(&set_type);
        let values_ptr = vm.object_of_type(&set_type);

        let mut key_chunks = Vec::new();
        let mut value_chunks = Vec::new();

        let mut changes = Vec::new();
        let mut dependencies = Vec::new();

        for node in content {
            if let NodeType::Pair {ref first, ref second} = node.kind {
                let intermediate = vm.execute(executors, first);
                let mut chunk = CollectionChunk::empty();

                for (path, address) in intermediate.result{
                    let kind = vm.get_object(&address).get_extension().first().unwrap();
                    let repr = Representant::new(address, *kind, Some(1), Some(1));
                    chunk.add_representant(path, repr);    
                }
                
                key_chunks.push(chunk);

                let mut intermediate = vm.execute(executors, second);
                let mut chunk = CollectionChunk::empty();

                for (path, address) in intermediate.result{
                    let kind = vm.get_object(&address).get_extension().first().unwrap();
                    let repr = Representant::new(address, *kind, Some(1), Some(1));
                    chunk.add_representant(path, repr);    
                }
                
                changes.append(&mut intermediate.changes);
                dependencies.append(&mut intermediate.dependencies);

                value_chunks.push(chunk);
            }
        }

        {
            let mut obj = vm.get_object_mut(&keys_ptr);
            obj.define_elements(key_chunks, Path::empty());
        }

        {
            let mut obj = vm.get_object_mut(&values_ptr);
            obj.define_elements(value_chunks, Path::empty());
        }

        {
            let mut obj = vm.get_object_mut(&dict_ptr);
            let keys_mapping = Mapping::simple(Path::empty(), keys_ptr);
            let values_mapping = Mapping::simple(Path::empty(), values_ptr);

            obj.assign_attribute("___keys".to_owned(), Path::empty(), keys_mapping);
            obj.assign_attribute("___values".to_owned(), Path::empty(), values_mapping);
        }

        let mapping = Mapping::simple(Path::empty(), dict_ptr);

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: mapping,
        }
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
        let type_name = "str".to_owned();

        let string_type = *vm.knowledge().get_type(&type_name).unwrap();

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

        let mapping = Mapping::simple(Path::empty(), string_ptr);

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec!(),
            result: mapping,
        }
    }
}

pub struct PythonInt { }

impl IntExecutor for PythonInt {
    fn execute(&self, env: Environment) -> ExecutionResult {
        let Environment { vm, .. } = env;
        let type_name = "int".to_owned();
        let pointer = vm.object_of_type(&type_name);

        let mapping = Mapping::simple(Path::empty(), pointer);

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![],
            result: mapping,
        }
    }
}

pub struct PythonFloat { }

impl FloatExecutor for PythonFloat {
    fn execute(&self, env: Environment) -> ExecutionResult {
        let Environment { vm, .. } = env;
        let type_name = "float".to_owned();
        let pointer = vm.object_of_type(&type_name);

        let mapping = Mapping::simple(Path::empty(), pointer);

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![],
            result: mapping,
        }
    }
}

fn make_collection(
    vm: &mut VirtualMachine, 
    executors: &Executors, 
    obj_ptr: Pointer,
    content: &[GastNode]) 
    -> ExecutionResult {

    if let Some(node) = content.first() {
        match node.kind {
            NodeType::Map { .. } => {
                collection_from_comprehension(vm, executors, obj_ptr, node)
            },
            _ => {
                collection_from_literal(vm, executors, obj_ptr, content)
            }
        }
    } else {
        collection_from_literal(vm, executors, obj_ptr, content)
    }
}

// different from normal generators because these make their own scope
fn collection_from_comprehension(
    vm: &mut VirtualMachine, 
    executors: &Executors, 
    obj_ptr: Pointer,
    content: &GastNode,
) -> ExecutionResult {

    let mut path = vm.current_path().clone();
    let node = PathNode::Frame(vm.current_node().clone(), 
        Some("comprehension".to_owned()), 
        0, 
        1);
    path.add_node(node);
    vm.push_path(path);

    let content_result = vm.execute(executors, content);
    let changes = content_result.changes;
    let dependencies = content_result.dependencies;

    let mut chunk = CollectionChunk::empty();
    for (path, address) in content_result.result {
        let obj = vm.get_object(&address);

        // todo, replace current node with the node of the generator
        for (el_path, el_address) in obj.get_any_element(vm.current_node()) {
            let mut new_path = path.clone();
            for node in el_path {
                new_path.add_node(node);
            }

            let kind = vm.get_object(&el_address).get_extension().first().unwrap();
            let repr = Representant::new(el_address, *kind, Some(1), None);
            chunk.add_representant(new_path, repr);  
        }
    }

    {
        let mut obj = vm.get_object_mut(&obj_ptr);
        obj.define_elements(vec!(chunk), Path::empty());
    }

    vm.merge_function(&changes);

    vm.pop_path();

    let mapping = Mapping::simple(Path::empty(), obj_ptr);

    ExecutionResult {
        flow: FlowControl::Continue,
        dependencies: dependencies,
        changes: changes,
        result: mapping,
    }
}

fn collection_from_literal(
    vm: &mut VirtualMachine, 
    executors: &Executors, 
    obj_ptr: Pointer,
    content: &[GastNode]) 
    -> ExecutionResult {

    let mut changes = Vec::new();
    let mut dependencies = Vec::new();

    let mut chunks = Vec::new();
    for node in content {
        let mut intermediate = vm.execute(executors, node);

        let mut chunk = CollectionChunk::empty();

        for (path, address) in intermediate.result{
            let kind = vm.get_object(&address).get_extension().first().unwrap();
            let repr = Representant::new(address, *kind, Some(1), Some(1));
            chunk.add_representant(path, repr);    
        }

        changes.append(&mut intermediate.changes);
        dependencies.append(&mut intermediate.dependencies);
        
        chunks.push(chunk);
    }

    {
        let mut obj = vm.get_object_mut(&obj_ptr);
        obj.define_elements(chunks, Path::empty());
    }

    let mapping = Mapping::simple(Path::empty(), obj_ptr);

    ExecutionResult {
        flow: FlowControl::Continue,
        dependencies: dependencies,
        changes: changes,
        result: mapping,
    }
}