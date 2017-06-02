use core::*;

use super::check_arg;

pub fn new_str_module() -> Module {
    let mut string = Module::new();
    define_format(&mut string);
    define_find(&mut string);
    define_upper(&mut string);
    define_lower(&mut string);
    define_isalpha(&mut string);
    string
}

fn define_format(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"method".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if !args.is_empty() {
                check_arg(vm, &args[0], "first", vec!("object"));
            }

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

            let path = vm.current_path().clone();
            vm.add_result(path, mapping);

            ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: vec!(),
                changes: vec!(),
                result: Mapping::new(),
            }
        };

        vm.set_callable(pointer, inner);

        pointer
    };
    
    module.add_part("format".to_owned(), Box::new(outer));
}

fn define_find(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"method".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let total_changes = Vec::new();
            let total_dependencies = Vec::new();

            let Environment { vm, .. } = env;

            if !args.is_empty() {
                check_arg(vm, &args[0], "first", vec!("str"));
            }

            let type_name = "int".to_owned();

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

            let path = vm.current_path().clone();
            vm.add_result(path, mapping);

            ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: total_dependencies,
                changes: total_changes,
                result: Mapping::new(),
            }
        };

        vm.set_callable(pointer, inner);

        pointer
    };
    
    module.add_part("find".to_owned(), Box::new(outer));
}

fn define_upper(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"method".to_owned());

        let inner = | env: Environment, _: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

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

            let path = vm.current_path().clone();
            vm.add_result(path, mapping);

            ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: vec!(),
                changes: vec!(),
                result: Mapping::new(),
            }
        };

        vm.set_callable(pointer, inner);

        pointer
    };
    
    module.add_part("upper".to_owned(), Box::new(outer));
}

fn define_lower(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"method".to_owned());

        let inner = | env: Environment, _: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

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

            let path = vm.current_path().clone();
            vm.add_result(path, mapping);

            ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: vec!(),
                changes: vec!(),
                result: Mapping::new(),
            }
        };

        vm.set_callable(pointer, inner);

        pointer
    };
    
    module.add_part("lower".to_owned(), Box::new(outer));
}

fn define_isalpha(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, _: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            let type_name = "bool".to_owned();
            let pointer = vm.object_of_type(&type_name);

            let mapping = Mapping::simple(Path::empty(), pointer);
            let path = vm.current_path().clone();
            vm.add_result(path, mapping);

            ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: vec!(AnalysisItem::Object(5)),
                changes: vec!(AnalysisItem::Object(5)),
                result: Mapping::new(),
            }
        };

        vm.set_callable(pointer, inner);

        pointer
    };

    module.add_part("isalpha".to_owned(), Box::new(outer));
}