#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
extern crate bidir_map;
extern crate term_painter;

extern crate rustc_serialize;
use rustc_serialize::json::Json;

pub mod core;
pub mod python;

pub use core::VirtualMachine;
use core::build;

use std::io::prelude::*;
use std::fs::File;
use core::Worker;
use core::Executors;


use python::*;

// todo implement for each builtin function
pub struct BuiltinFunction {

}

impl BuiltinFunction {
    // fn call(&self, kb: &mut KnowledgeBase, args: [&Object]);
}

pub struct FunctionDefinition {

}
/// type aliases

type Type = i16;

// todo change to enum; Data -> &Object / Code -> Callable
pub type Pointer = i16;
type TypePointer = i16;

fn main() {
    let worker = Worker::new();
    test_vm();
    let _ = worker.finalize();
    // test_collection();
}



fn test_vm() {
    let executors = Executors {
        assign: Some(Box::new(PythonAssign {})),
        attribute: Some(Box::new(PythonAttribute {})),
        binop: Some(Box::new(PythonBinOp {})),
        boolop: Some(Box::new(PythonBoolOp {})),
        block: Some(Box::new(PythonBlock {})),
        boolean: Some(Box::new(PythonBoolean {})),
        conditional: Some(Box::new(PythonConditional {})),
        declaration: None,
        float: Some(Box::new(PythonFloat {})),
        identifier: Some(Box::new(PythonIdentifier {})),
        int: Some(Box::new(PythonInt {})),
        string: Some(Box::new(PythonString {})),
        while_loop: Some(Box::new(PythonWhile {})),
        break_loop: Some(Box::new(PythonBreak {})),
        continue_loop: Some(Box::new(PythonContinue {})),
        list: Some(Box::new(PythonList {})),
        sequence: Some(Box::new(PythonTuple {})),
        index: Some(Box::new(PythonIndex {})),
        set: Some(Box::new(PythonSet {})),
        dict: Some(Box::new(PythonDict {})),
        generator: Some(Box::new(PythonGenerator {})),
        filter: Some(Box::new(PythonFilter {})),
        map: Some(Box::new(PythonMap {})),
        andthen: Some(Box::new(PythonAndThen {})),
        foreach: Some(Box::new(PythonFor {})),
        call: Some(Box::new(PythonCall {})),
        method: Some(Box::new(PythonMethod {})),
        import: Some(Box::new(PythonImport {})),
        negate: Some(Box::new(PythonNegate {})),
        unop: Some(Box::new(PythonUnOp {})),
        slice: Some(Box::new(PythonSlice {})),
        function: Some(Box::new(PythonFunction {})),
    };

    let mut s = String::new();

    let _ = match File::open("input.json") {
        Ok(mut file) => file.read_to_string(&mut s),
        Err(why) => panic!("{:?}", why),
    };

    let json = Json::from_str(&s).unwrap();
    let stuff = build(&json);

    let mut vm = VirtualMachine::new();

    // builtins
    vm.new_scope();

    vm.declare_simple_type(&"object".to_owned());
    vm.declare_sub_type(&executors, &"NoneType".to_owned(), &"object".to_owned());
    vm.declare_new_constant(&"None".to_owned(), &"NoneType".to_owned());

    // magical variables, used internally
    vm.declare_simple_type(&"hidden".to_owned());
    vm.declare_new_constant(&"___implicit".to_owned(), &"hidden".to_owned()); // at address 5
    vm.declare_new_constant(&"___result".to_owned(), &"hidden".to_owned());

    // 
    vm.declare_sub_type(&executors, &"function".to_owned(), &"object".to_owned());
    vm.declare_sub_type(&executors, &"method".to_owned(), &"function".to_owned());
    vm.declare_sub_type(&executors, &"module".to_owned(), &"object".to_owned());

    vm.declare_sub_type(&executors, &"number".to_owned(), &"object".to_owned());
    vm.declare_sub_type(&executors, &"int".to_owned(), &"number".to_owned());
    vm.declare_sub_type(&executors, &"float".to_owned(), &"number".to_owned());
    vm.declare_sub_type(&executors, &"bool".to_owned(), &"int".to_owned());
    vm.declare_new_constant(&"True".to_owned(), &"bool".to_owned());
    vm.declare_new_constant(&"False".to_owned(), &"bool".to_owned());

    // cpython doesn't really have a collection type, still convenient
    // all iterable things
    vm.declare_sub_type(&executors, &"collection".to_owned(), &"object".to_owned());

    vm.declare_sub_type(&executors, &"set".to_owned(), &"collection".to_owned());
    vm.declare_sub_type(&executors, &"dict".to_owned(), &"collection".to_owned());

    // sequences have are ordered
    vm.declare_sub_type(&executors, &"sequence".to_owned(), &"collection".to_owned());

    vm.declare_sub_type(&executors, &"immutable_sequence".to_owned(), &"sequence".to_owned());
    vm.declare_sub_type(&executors, &"string".to_owned(), &"immutable_sequence".to_owned());
    vm.declare_sub_type(&executors, &"tuple".to_owned(), &"immutable_sequence".to_owned());
    vm.declare_sub_type(&executors, &"byte".to_owned(), &"immutable_sequence".to_owned());

    vm.declare_sub_type(&executors, &"mutable_sequence".to_owned(), &"sequence".to_owned());
    vm.declare_sub_type(&executors, &"list".to_owned(), &"mutable_sequence".to_owned());
    vm.declare_sub_type(&executors, &"byte_array".to_owned(), &"mutable_sequence".to_owned());

    {
        let mut kb = vm.knowledge_base();
        kb.add_arithmetic_type("number", "+");
        kb.add_arithmetic_type("number", "-");
        kb.add_arithmetic_type("number", "/");
        kb.add_arithmetic_type("number", "*");
        kb.add_arithmetic_type("number", "//");
        kb.add_arithmetic_type("number", "**");
        kb.add_arithmetic_type("number", "%");

        // ints have their own implementation
        // avoid coercion to float/number
        kb.add_arithmetic_type("int", "+");
        kb.add_arithmetic_type("int", "-");
        //kb.add_arithmetic_type("int", "/");
        kb.add_arithmetic_type("int", "*");
        kb.add_arithmetic_type("int", "//");
        kb.add_arithmetic_type("int", "**");
        kb.add_arithmetic_type("int", "%");

        kb.add_arithmetic_type("bool", "or");
        kb.add_arithmetic_type("bool", "and");

        kb.add_arithmetic_type("number", "<");
        kb.add_arithmetic_type("number", ">");
        kb.add_arithmetic_type("number", "<=");
        kb.add_arithmetic_type("number", ">=");

        kb.add_arithmetic_type("object", "is");
        kb.add_arithmetic_type("object", "is not");
        kb.add_arithmetic_type("object", "==");
        kb.add_arithmetic_type("object", "!=");

        // == on None is _not_ a valid operation
        kb.add_arithmetic_type("NoneType", "is");
        kb.add_arithmetic_type("NoneType", "is not");

        kb.add_arithmetic_type("collection", "in");
        kb.add_arithmetic_type("collection", "not in");

        kb.add_arithmetic_type("list", "+");
        kb.add_arithmetic_type("tuple", "+");
        kb.add_arithmetic_type("string", "+");

        kb.add_arithmetic_type("set", "-");
    }

    define_modules(&mut vm);

    vm.import(&executors, &"builtin".to_owned(), &vec!(), &None);
    vm.import(&executors, &"string".to_owned(), &vec!(), &Some("string".to_owned()));

    // global scope
    vm.new_scope();

    vm.execute(&executors, &stuff);
}

fn define_modules(vm: &mut VirtualMachine) {
    let builtins = new_builtin_module();
    vm.insert_module("builtin".to_owned(), builtins);
    
    let math = new_math_module();
    vm.insert_module("math".to_owned(), math);

    let string = new_string_module();
    vm.insert_module("string".to_owned(), string);

/*
    define_int_cast(vm);
    define_float_cast(vm);
    define_abs(vm);
    define_all(vm);
    define_any(vm);
    define_ord(vm);
    define_len(vm);
    define_repr(vm);
    define_string_cast(vm);
    define_round(vm);
    //define_input(vm);
    define_print(vm);
    define_string_format(vm);
    */
}

/*





fn define_float_cast(vm: &mut VirtualMachine) {
    let ptr = vm.knowledge().get_type(&"float".to_owned()).unwrap().clone();

    let fun = | env: Environment, args: Vec<Mapping>, _: &HashMap<String, GastNode> | {
        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();

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

    vm.set_callable(ptr, fun);
}

fn define_string_cast(vm: &mut VirtualMachine) {
    let fun = | env: Environment, args: Vec<Mapping>, _: &HashMap<String, GastNode> | {
        let Environment { vm, .. } = env;

        if args.len() > 0 {
            check_arg(vm, &args[0], "first", vec!("object", "NoneType"));
        }

        let type_name = "string".to_owned();
        let pointer = vm.object_of_type(&type_name);

        let mapping = Mapping::simple(Path::empty(), pointer.clone());
        let path = vm.current_path().clone();
        vm.set_result(path, mapping);

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec!(),
            changes: vec!(),
            result: Mapping::new(),
        };

        execution_result
    };

    vm.define_function("str".to_owned(), fun);
}



fn define_abs(vm: &mut VirtualMachine) {
    let fun = | env: Environment, args: Vec<Mapping>, _: &HashMap<String, GastNode> | {
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
            dependencies: vec!(),
            changes: vec!(),
            result: Mapping::new(),
        };

        execution_result
    };

    vm.define_function("abs".to_owned(), fun);
}

fn define_round(vm: &mut VirtualMachine) {
    let fun = | env: Environment, args: Vec<Mapping>, _: &HashMap<String, GastNode> | {
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
            dependencies: vec!(),
            changes: vec!(),
            result: Mapping::new(),
        };

        execution_result
    };

    vm.define_function("round".to_owned(), fun);
}

fn define_len(vm: &mut VirtualMachine) {
    let fun = | env: Environment, args: Vec<Mapping>, _: &HashMap<String, GastNode> | {
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
            dependencies: vec!(),
            changes: vec!(),
            result: Mapping::new(),
        };

        execution_result
    };

    vm.define_function("len".to_owned(), fun);
}

fn define_repr(vm: &mut VirtualMachine) {
    let fun = | env: Environment, args: Vec<Mapping>, _: &HashMap<String, GastNode> | {
        let Environment { vm, .. } = env;

        if args.len() > 0 {
            check_arg(vm, &args[0], "first", vec!("object", "NoneType"));
        }
        
        let type_name = "string".to_owned();
        let pointer = vm.object_of_type(&type_name);

        let mapping = Mapping::simple(Path::empty(), pointer.clone());
        let path = vm.current_path().clone();
        vm.set_result(path, mapping);

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec!(),
            changes: vec!(),
            result: Mapping::new(),
        };

        execution_result
    };

    vm.define_function("repr".to_owned(), fun);
}

fn define_ord(vm: &mut VirtualMachine) {
    let fun = | env: Environment, args: Vec<Mapping>, _: &HashMap<String, GastNode> | {
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
            dependencies: vec!(),
            changes: vec!(),
            result: Mapping::new(),
        };

        execution_result
    };

    vm.define_function("ord".to_owned(), fun);
}

fn define_all(vm: &mut VirtualMachine) {
    let fun = | env: Environment, args: Vec<Mapping>, _: &HashMap<String, GastNode> | {
        let Environment { vm, .. } = env;

        if args.len() > 0 {
            check_arg(vm, &args[0], "first", vec!("collection"));
        }
    
        let type_name = "bool".to_owned();
        let pointer = vm.object_of_type(&type_name);

        let mapping = Mapping::simple(Path::empty(), pointer.clone());
        let path = vm.current_path().clone();
        vm.set_result(path, mapping);

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec!(),
            changes: vec!(),
            result: Mapping::new(),
        };

        execution_result
    };

    vm.define_function("all".to_owned(), fun);
}

fn define_any(vm: &mut VirtualMachine) {
    let fun = | env: Environment, args: Vec<Mapping>, _: &HashMap<String, GastNode> | {
        let Environment { vm, .. } = env;

        if args.len() > 0 {
            check_arg(vm, &args[0], "first", vec!("collection"));
        }
    
        let type_name = "bool".to_owned();
        let pointer = vm.object_of_type(&type_name);

        let mapping = Mapping::simple(Path::empty(), pointer.clone());
        let path = vm.current_path().clone();
        vm.set_result(path, mapping);

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec!(),
            changes: vec!(),
            result: Mapping::new(),
        };

        execution_result
    };

    vm.define_function("any".to_owned(), fun);
}*/