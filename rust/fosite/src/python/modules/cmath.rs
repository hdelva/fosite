use core::*;

use super::check_arg;

pub fn new_cmath_module() -> Module {
    let mut cmath = Module::new();
    
    define_cos(&mut cmath);
    define_sin(&mut cmath);
    define_tan(&mut cmath);

    cmath
}

fn define_sin(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if !args.is_empty() {
                check_arg(vm, &args[0], "first", vec!("number"));
            }

            let type_name = "float".to_owned();
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

    module.add_part("sin".to_owned(), Box::new(outer));
}

fn define_cos(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if !args.is_empty() {
                check_arg(vm, &args[0], "first", vec!("number"));
            }

            let type_name = "float".to_owned();
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

    module.add_part("cos".to_owned(), Box::new(outer));
}

fn define_tan(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"function".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if !args.is_empty() {
                check_arg(vm, &args[0], "first", vec!("number"));
            }

            let type_name = "float".to_owned();
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

    module.add_part("tan".to_owned(), Box::new(outer));
}