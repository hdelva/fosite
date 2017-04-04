use core::*;

use super::check_arg;

pub fn new_builtin_module() -> Module {
    let mut builtin = Module::new();

    define_int_cast(&mut builtin);
    define_float_cast(&mut builtin);
    define_str_cast(&mut builtin);
    
    define_input(&mut builtin);
    define_print(&mut builtin);

    define_abs(&mut builtin);
    define_round(&mut builtin);
    define_ord(&mut builtin);
    define_len(&mut builtin);

    define_range(&mut builtin);
    
    builtin
}

fn define_int_cast(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let ptr = vm.knowledge().get_type(&"int".to_owned()).unwrap().clone();

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let total_changes = Vec::new();
            let total_dependencies = Vec::new();

            let Environment { vm, .. } = env;

            if args.len() > 0 {
                check_arg(vm, &args[0], "first", vec!("number", "string"));
            }
            
            let type_name = "int".to_owned();
            let pointer = vm.object_of_type(&type_name);

            let mapping = Mapping::simple(Path::empty(), pointer.clone());
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

        vm.set_callable(ptr, inner);

        ptr
    };

    module.add_part("int".to_owned(), Box::new(outer));
}

fn define_str_cast(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"method".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let total_changes = Vec::new();
            let total_dependencies = Vec::new();

            let Environment { vm, .. } = env;

            if args.len() > 0 {
                check_arg(vm, &args[0], "first", vec!("object"));
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
    
    module.add_part("str".to_owned(), Box::new(outer));
}

fn define_float_cast(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let ptr = vm.knowledge().get_type(&"float".to_owned()).unwrap().clone();

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let total_changes = Vec::new();
            let total_dependencies = Vec::new();

            let Environment { vm, .. } = env;

            if args.len() > 0 {
                check_arg(vm, &args[0], "first", vec!("number", "string"));
            }
            
            let type_name = "float".to_owned();
            let pointer = vm.object_of_type(&type_name);

            let mapping = Mapping::simple(Path::empty(), pointer.clone());
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

        vm.set_callable(ptr, inner);

        ptr
    };

    module.add_part("float".to_owned(), Box::new(outer));
}

fn define_input(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if args.len() > 0 {
                check_arg(vm, &args[0], "first", vec!("object", "NoneType"));
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
                dependencies: vec!(AnalysisItem::Object(5)),
                changes: vec!(AnalysisItem::Object(5)),
                result: Mapping::new(),
            };

            execution_result
        };

        vm.set_callable(pointer.clone(), inner);

        pointer
    };

    module.add_part("input".to_owned(), Box::new(outer));
}

fn define_print(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            for arg in args.iter() {
                check_arg(vm, arg, "any", vec!("object", "NoneType"));
            }

            let execution_result = ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: vec!(AnalysisItem::Object(5)),
                changes: vec!(AnalysisItem::Object(5)),
                result: Mapping::new(),
            };

            execution_result
        };

        vm.set_callable(pointer.clone(), inner);

        pointer
    };

    module.add_part("print".to_owned(), Box::new(outer));
}

fn define_abs(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if args.len() > 0 {
                check_arg(vm, &args[0], "first", vec!("number"));
            }

            let type_name = "int".to_owned();
            let pointer = vm.object_of_type(&type_name);

            let mapping = Mapping::simple(Path::empty(), pointer.clone());
            let path = vm.current_path().clone();
            vm.set_result(path, mapping);

            let execution_result = ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: vec!(AnalysisItem::Object(5)),
                changes: vec!(AnalysisItem::Object(5)),
                result: Mapping::new(),
            };

            execution_result
        };

        vm.set_callable(pointer.clone(), inner);

        pointer
    };

    module.add_part("abs".to_owned(), Box::new(outer));
}


fn define_len(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if args.len() > 0 {
                check_arg(vm, &args[0], "first", vec!("collection"));
            }

            let type_name = "int".to_owned();
            let pointer = vm.object_of_type(&type_name);

            let mapping = Mapping::simple(Path::empty(), pointer.clone());
            let path = vm.current_path().clone();
            vm.set_result(path, mapping);

            let execution_result = ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: vec!(AnalysisItem::Object(5)),
                changes: vec!(AnalysisItem::Object(5)),
                result: Mapping::new(),
            };

            execution_result
        };

        vm.set_callable(pointer.clone(), inner);

        pointer
    };

    module.add_part("len".to_owned(), Box::new(outer));
}

fn define_round(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if args.len() > 0 {
                check_arg(vm, &args[0], "first", vec!("number"));
            }

            let type_name = "int".to_owned();
            let pointer = vm.object_of_type(&type_name);

            let mapping = Mapping::simple(Path::empty(), pointer.clone());
            let path = vm.current_path().clone();
            vm.set_result(path, mapping);

            let execution_result = ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: vec!(AnalysisItem::Object(5)),
                changes: vec!(AnalysisItem::Object(5)),
                result: Mapping::new(),
            };

            execution_result
        };

        vm.set_callable(pointer.clone(), inner);

        pointer
    };

    module.add_part("round".to_owned(), Box::new(outer));
}

fn define_ord(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if args.len() > 0 {
                check_arg(vm, &args[0], "first", vec!("string"));
            }

            let type_name = "int".to_owned();
            let pointer = vm.object_of_type(&type_name);

            let mapping = Mapping::simple(Path::empty(), pointer.clone());
            let path = vm.current_path().clone();
            vm.set_result(path, mapping);

            let execution_result = ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: vec!(AnalysisItem::Object(5)),
                changes: vec!(AnalysisItem::Object(5)),
                result: Mapping::new(),
            };

            execution_result
        };

        vm.set_callable(pointer.clone(), inner);

        pointer
    };

    module.add_part("ord".to_owned(), Box::new(outer));
}

fn define_range(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if args.len() > 1 {
                check_arg(vm, &args[0], "first", vec!("int"));
                check_arg(vm, &args[1], "second", vec!("int"));
            }

            let list_type_name = "list".to_owned();

            let int_type_name = "int".to_owned();
            let int_type = vm.knowledge().get_type(&int_type_name).unwrap().clone();

            let list_ptr = vm.object_of_type(&list_type_name);
            let int_ptr = vm.object_of_type(&int_type_name);

            {
                let mut list_object = vm.get_object_mut(&list_ptr);
                let repr = Representant::new(int_ptr, int_type, None, None);
                let mut chunk = CollectionChunk::empty();
                chunk.add_representant(Path::empty(), repr);
                list_object.define_elements(vec!(chunk), Path::empty());
            }

            let mapping = Mapping::simple(Path::empty(), list_ptr.clone());
            
            let path = vm.current_path().clone();
            vm.set_result(path, mapping);

            let execution_result = ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: vec!(AnalysisItem::Object(5)),
                changes: vec!(AnalysisItem::Object(5)),
                result: Mapping::new(),
            };

            execution_result
        };

        vm.set_callable(pointer.clone(), inner);

        pointer
    };

    module.add_part("range".to_owned(), Box::new(outer));
}