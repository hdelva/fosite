use core::*;

use super::check_arg;

pub fn new_math_module() -> Module {
    let mut math = Module::new();
    
    define_cos(&mut math);
    define_sin(&mut math);
    define_radians(&mut math);
    define_floor(&mut math);

    define_pi(&mut math);
    define_e(&mut math);

    math
}

fn define_sin(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if args.len() > 0 {
                check_arg(vm, &args[0], "first", vec!("number"));
            }

            let type_name = "float".to_owned();
            let pointer = vm.object_of_type(&type_name);

            let mapping = Mapping::simple(Path::empty(), pointer.clone());
            let path = vm.current_path().clone();
            vm.add_result(path, mapping);

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

    module.add_part("sin".to_owned(), Box::new(outer));
}

fn define_cos(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if args.len() > 0 {
                check_arg(vm, &args[0], "first", vec!("number"));
            }

            let type_name = "float".to_owned();
            let pointer = vm.object_of_type(&type_name);

            let mapping = Mapping::simple(Path::empty(), pointer.clone());
            let path = vm.current_path().clone();
            vm.add_result(path, mapping);

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

    module.add_part("cos".to_owned(), Box::new(outer));
}

fn define_radians(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if args.len() > 0 {
                check_arg(vm, &args[0], "first", vec!("number"));
            }

            let type_name = "float".to_owned();
            let pointer = vm.object_of_type(&type_name);

            let mapping = Mapping::simple(Path::empty(), pointer.clone());
            let path = vm.current_path().clone();
            vm.add_result(path, mapping);

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

    module.add_part("radians".to_owned(), Box::new(outer));
}

fn define_floor(module: &mut Module) {
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
            vm.add_result(path, mapping);

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

    module.add_part("floor".to_owned(), Box::new(outer));
}


fn define_pi(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"float".to_owned());

        pointer
    };

    module.add_part("pi".to_owned(), Box::new(outer));
}

fn define_e(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"float".to_owned());

        pointer
    };

    module.add_part("e".to_owned(), Box::new(outer));
}