#![allow(dead_code)]

extern crate bidir_map;
extern crate carboxyl;

pub mod core;
use core::VirtualMachine;
use core::GastNode;
use core::NodeType;

use std::thread;
use std::time::Duration;

use std::io::prelude::*;
use std::fs::File;


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

// Giving the compiler something to do
fn main() {
	let mut s = String::new();
	
	match File::open("output.json") {
		Ok(mut file) => file.read_to_string(&mut s),
		Err(why) => panic!("{:?}", why),
	};
	
	println!("{}", s);
	
    let sink = carboxyl::Sink::new();
    let stream = sink.stream();
    let events = stream.events();
    thread::spawn ( move || {
            for event in events {
                println!("{:?}", event);
            }
        });


    {
        let mut vm = VirtualMachine::new(sink.clone());

        vm.new_context();

        vm.declare_simple_type(&"number".to_owned());
        vm.declare_simple_type(&"Stub".to_owned());

        test1(&mut vm);
        println!("");

        test2(&mut vm);
        println!("");

        test3(&mut vm);

        thread::sleep(Duration::from_millis(4000))
    }
}

fn test1(vm: &mut VirtualMachine) {
    let x = GastNode::new(0, NodeType::Identifier { name: "x".to_owned() });
    let value = Box::new(GastNode::new(1, NodeType::Number { value: 5 }));
    let assignment = GastNode::new(2, NodeType::Assignment {
        targets: vec![x],
        value: value,
    });

    // executing x = 5
    //println!("Executing \"x = 5\"");
    let result = vm.execute(&assignment);
    //println!("{:?}", result);

    vm.inspect_identifier(&"number".to_owned());
    vm.inspect_identifier(&"x".to_owned());
}


fn test2(vm: &mut VirtualMachine) {
    let declaration = GastNode::new(3, NodeType::Declaration {
        id: "z".to_owned(),
        kind: "Stub".to_owned(),
    });
    vm.execute(&declaration);

    // jam a placeholder in there
    let address = 3;
    let child_address = vm.memory.new_object();
    {
        let mut object = vm.memory.get_object_mut(&address);
        object.enable_iteration(child_address);
    }

    let x = GastNode::new(4, NodeType::Identifier { name: "x".to_owned()});
    let y = GastNode::new(5, NodeType::Identifier { name: "y".to_owned()});
    let z = GastNode::new(6, NodeType::Identifier { name: "z".to_owned()});

    let target = GastNode::new(7, NodeType::List { content: vec![x, y]});
    let assignment = GastNode::new(8, NodeType::Assignment {
        targets: vec![target],
        value: Box::new(z),
    });

    //println!("Executing \"x, y = z\"");
    let result = vm.execute(&assignment);
    //println!("{:?}", result);

    vm.inspect_identifier(&"x".to_owned());
    vm.inspect_identifier(&"y".to_owned());
}

fn test3(vm: &mut VirtualMachine) {
    let parent = GastNode::new(9, NodeType::Identifier { name: "x".to_owned()});
    let attribute = GastNode::new(10, NodeType::Attribute {
        parent: Box::new(parent),
        attribute: "attribute".to_owned(),
    });

    let value = Box::new(GastNode::new(11, NodeType::Number { value: 5 }));
    let assignment = GastNode::new( 12, NodeType::Assignment {
        targets: vec![attribute],
        value: value,
    });

    // executing x.attribute = 5
    //println!("Executing \"x.attribute = 5\"");
    let result = vm.execute(&assignment);
    //println!("{:?}", result);

}