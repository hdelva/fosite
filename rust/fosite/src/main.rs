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
//use core::Collection;
//use core::Representant;
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
/*
fn test_collection() {
    let mut collection = Collection::empty();
    let mut definition = vec![];
    definition.push(Representant::new(1, 1));
    definition.push(Representant::new(2, 1));
    definition.push(Representant::new(3, 0));
    collection.define(definition);

    println!("{:?}", collection.get_first_n(1));
    println!("{:?}", collection.get_last_n(1));
    println!("{:?}", collection.get_first_n(2));
    println!("{:?}", collection.get_last_n(2));

    println!("");

    collection.prepend(Representant::new(4, 4), 0, 2);
    println!("{:?}", collection.get_first_n(3));

    println!("");

    println!("{:?}", collection.slice(1, 1));
}
*/


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

    // global scope
    vm.new_scope();

    vm.execute(&executors, &stuff);
}
