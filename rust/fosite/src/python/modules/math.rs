use core::*;

use super::check_arg;

use std::collections::HashMap;

pub fn new_math_module(vm: &mut VirtualMachine) -> Module {
    let mut math = Module::new();
    
    define_cos(&mut math, vm);
    define_floor(&mut math, vm);

    define_pi(&mut math, vm);
    define_e(&mut math, vm);

    math
}

fn define_cos(module: &mut Module, vm: &mut VirtualMachine) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: &HashMap<String, GastNode> | {
            let Environment { vm, .. } = env;

            if args.len() > 0 {
                check_arg(vm, &args[0], "first", vec!("number"));
            }

            let type_name = "float".to_owned();
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

    module.add_part("cos".to_owned(), Box::new(outer));
}

fn define_floor(module: &mut Module, vm: &mut VirtualMachine) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: &HashMap<String, GastNode> | {
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

    module.add_part("floor".to_owned(), Box::new(outer));
}


fn define_pi(module: &mut Module, vm: &mut VirtualMachine) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"float".to_owned());

        pointer
    };

    module.add_part("pi".to_owned(), Box::new(outer));
}

fn define_e(module: &mut Module, vm: &mut VirtualMachine) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"float".to_owned());

        pointer
    };

    module.add_part("e".to_owned(), Box::new(outer));
}