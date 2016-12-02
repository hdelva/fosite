#![allow(dead_code)]
#![feature(box_syntax)]

extern crate bidir_map;
extern crate carboxyl;

pub mod core;
use core::VirtualMachine;
use core::GastNode;

use std::thread;
use std::time::Duration;

pub type GastID = i16;

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

struct Slave {
    events: carboxyl::stream::Events,
}

impl Slave {
    fn new(events: carboxyl::stream::Events) -> Slave {
        Slave {
            events: events,
        }
    }

    fn work(&mut self, events:carboxyl::stream::Events) {

    }
}


// Giving the compiler something to do
fn main() {
    let sink = carboxyl::Sink::new();
    let stream = sink.stream();
    let mut events = stream.events();
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
    let x = GastNode::Identifier { name: "x".to_owned() };
    let value = Box::new(GastNode::Number { value: 5 });
    let assignment = GastNode::Assignment {
        targets: vec![x],
        value: value,
    };

    // executing x = 5
    //println!("Executing \"x = 5\"");
    let result = vm.execute(&assignment);
    //println!("{:?}", result);

    vm.inspect_identifier(&"number".to_owned());
    vm.inspect_identifier(&"x".to_owned());
}


fn test2(vm: &mut VirtualMachine) {
    let declaration = GastNode::Declaration {
        id: "z".to_owned(),
        kind: "Stub".to_owned(),
    };
    vm.execute(&declaration);

    // jam a placeholder in there
    let address = 3;
    let child_address = vm.memory.new_object();
    {
        let mut object = vm.memory.get_object_mut(&address);
        object.enable_iteration(child_address);
    }

    let x = GastNode::Identifier { name: "x".to_owned() };
    let y = GastNode::Identifier { name: "y".to_owned() };
    let z = GastNode::Identifier { name: "z".to_owned() };

    let target = GastNode::List { content: vec![x, y] };
    let assignment = GastNode::Assignment {
        targets: vec![target],
        value: Box::new(z),
    };

    //println!("Executing \"x, y = z\"");
    let result = vm.execute(&assignment);
    //println!("{:?}", result);

    vm.inspect_identifier(&"x".to_owned());
    vm.inspect_identifier(&"y".to_owned());
}

fn test3(vm: &mut VirtualMachine) {
    let parent = GastNode::Identifier { name: "x".to_owned() };
    let attribute = GastNode::Attribute {
        parent: Box::new(parent),
        attribute: "attribute".to_owned(),
    };

    let value = Box::new(GastNode::Number { value: 5 });
    let assignment = GastNode::Assignment {
        targets: vec![attribute],
        value: value,
    };

    // executing x.attribute = 5
    //println!("Executing \"x.attribute = 5\"");
    let result = vm.execute(&assignment);
    //println!("{:?}", result);

}
