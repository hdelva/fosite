use core::*;

use super::check_arg;

use std::collections::HashMap;

pub fn new_string_module(vm: &mut VirtualMachine) -> Module {
    let mut string = Module::new();
    define_format(&mut string, vm);
    string
}

fn define_format(module: &mut Module, vm: &mut VirtualMachine) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"method".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: &HashMap<String, GastNode> | {
            let mut total_changes = Vec::new();
            let mut total_dependencies = Vec::new();

            let Environment { vm, .. } = env;

            if args.len() > 0 {
                check_arg(vm, &args[0], "first", vec!("number", "string"));
            }

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

            let path = vm.current_path().clone();
            vm.set_result(path, mapping);

            let execution_result = ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: total_dependencies,
                changes: total_changes,
                result: Mapping::new(),
            };

            execution_result
        };

        vm.set_callable(pointer.clone(), inner);

        pointer
    };
    
    module.add_part("format".to_owned(), Box::new(outer));
}