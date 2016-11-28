#![allow(dead_code)]
#![feature(box_syntax)]

extern crate bidir_map;


use std::collections::HashSet;

pub mod core;
use core::Memory;
use core::KnowledgeBase;
use core::result::*;
use core::Context;
use core::Mapping;
use core::Assumption;

pub type GastID = i16;


//todo implement for each builtin function
pub struct BuiltinFunction {

}

impl BuiltinFunction {
    //fn call(&self, kb: &mut KnowledgeBase, args: [&Object]);
}

pub struct FunctionDefinition {

}


//todo put these somewhere?
/*
    /// change things in the knowledge base
    //todo remove these
    fn link_objects(&mut self, parent_address: &Pointer, child_address: &Pointer) {
        match self.objects.get_mut(child_address) {
            Some(child) => {
                child.set_parent(parent_address.clone());
            },
            None => panic!("Invalid Pointer Value")
        }
    }

    fn limit_types(&mut self, address: &Pointer, limits: &HashSet<Type>) {
        match self.objects.get_mut(address) {
            Some(object) => {
                object.limit_types(limits);
            },
            None => panic!("Invalid Pointer Value")
        }
    }
*/


/// type aliases

type Type = i16;

//todo change to enum; Data -> &Object / Code -> Callable
pub type Pointer = i16;
type TypePointer = i16;

/// structs
enum GastNode {
    Identifier {name: String},
    Declaration {id: String, kind: String},
    Assignment {targets: Vec<GastNode>, value: Box<GastNode>},
    Number {value: i32},
    String {value: String},
    List {content: Vec<Box<GastNode>>},
    Sequence {content: Vec<Box<GastNode>>},
}

struct VirtualMachine {
    // instruction queue
    // call stack
    contexts: Vec<Context>,
    memory: Memory,
    knowledge_base: KnowledgeBase,
}

impl VirtualMachine {
    fn new() -> VirtualMachine {
        let memory = Memory::new();
        let knowledge = KnowledgeBase::new();
        VirtualMachine {
            contexts: Vec::new(),
            memory: memory,
            knowledge_base: knowledge,
        }
    }

    //todo is this still necessary
    /*
    fn merge_types(&mut self, first: &Pointer, second: &Pointer) {
        let first_object = self.memory.get_object(first);
        let second_object = self.memory.get_object(second);
    }*/

    pub fn execute(&mut self, node: &GastNode) -> ExecutionResult {
        match node {
            &GastNode::Number {ref value} => self.number(),
            &GastNode::Identifier {ref name} => self.load_identifier(name),
            &GastNode::String {ref value} => self.string(),
            &GastNode::Declaration {ref id, ref kind} => self.declaration(id, kind),
            &GastNode::Assignment {ref targets, ref value} => self.assign(targets, value),
            _ => panic!("Unsupported Operation"),
        }
    }

    fn string(&mut self) -> ExecutionResult {
        let pointer = self.memory.new_object();
        let object = self.memory.get_object_mut(&pointer);
        let type_name = "string".to_owned();
        let type_pointer = self.knowledge_base.get_type(&type_name);

        match type_pointer {
            Some(address) => {
                object.extends(address.clone());
            },
            _ => panic!("system isn't properly initialized")
        }

        let result = Result {
            assumption: Assumption::None,
            value: pointer.clone(),
        };

        let execution_result = ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec!(),
            changes: vec!(),
            results: vec!(result),
        };

        return execution_result
    }

    fn declaration(&mut self, name: &String, type_name: &String) -> ExecutionResult {
        let pointer = self.memory.new_object();
        let object = self.memory.get_object_mut(&pointer);
        let type_pointer = self.knowledge_base.get_type(type_name);

        match type_pointer {
            Some(address) => {
                object.extends(address.clone());
            },
            _ => panic!("declaration type does not exist")
        }

        let mut possibilities = HashSet::new();
        possibilities.insert(pointer.clone());

        match self.contexts.last_mut() {
            Some(mut last) => {
                let scope = last.get_public_scope_mut();
                scope.invalidate_mappings(name);
                let mapping = VirtualMachine::create_simple_mapping(pointer);
                scope.add_mapping(name, mapping);
            },
            _ => panic!("No Execution Contexts")
        }

        let result = Result {
            assumption: Assumption::None,
            value: -1, //todo change to 'python' None
        };

        let execution_result = ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec!(),
            changes: vec!(Change::Identifier {name: name.clone()} ),
            results: vec!(result),
        };

        return execution_result
    }

    fn number(&mut self) -> ExecutionResult {
        let pointer = self.memory.new_object();
        let object = self.memory.get_object_mut(&pointer);
        let type_name = "number".to_owned();
        let type_pointer = self.knowledge_base.get_type(&type_name);

        match type_pointer {
            Some(address) => {
                object.extends(address.clone());
            },
            _ => panic!("system isn't properly initialized")
        }

        let result = Result {
            assumption: Assumption::None,
            value: pointer.clone(),
        };

        let execution_result = ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec!(),
            changes: vec!(),
            results: vec!(result),
        };

        return execution_result
    }

    fn load_identifier(&mut self, name: &String) -> ExecutionResult {
        let mut candidate = None;

        for context in self.contexts.iter().rev() {
            candidate = context.get_public_scope().resolve_identifier(&name);

            if candidate.is_some() {
                break;
            }
        }

        match candidate {
            Some(mappings) => {
                let mut results = Vec::new();

                for mapping in mappings {
                    let assumption = mapping.assumption.clone();
                    let address = mapping.address;

                    results.push(Result {assumption: assumption, value: address} )
                }

                let execution_result = ExecutionResult::Success {
                    flow: FlowControl::Continue,
                    dependencies: vec!(name.clone()),
                    changes: vec!(),
                    results: results,
                };

                return execution_result
            },
            _ => panic!("Invalid Identifier")
        }
    }


    fn assign(&mut self, targets: &Vec<GastNode>, value: &GastNode) -> ExecutionResult {
        let value_execution = self.execute(value);

        match value_execution {
            ExecutionResult::Success{flow, dependencies, mut changes, results} => {
                let mut mappings = Vec::new();

                for result in results {
                    mappings.push( Mapping{assumption: result.assumption, address: result.value} )
                }

                for target in targets {
                    let partial_result = self.assign_to_target(target, &mappings);

                    match partial_result {
                        ExecutionResult::Success {
                            flow: t_flow,
                            dependencies: t_dependencies,
                            changes: mut t_changes,
                            results: t_results} => changes.append(&mut t_changes),
                        _ => panic!("bad shit")
                    }
                }

                let values = vec!(Result {
                    assumption: Assumption::None,
                    value: -1, //todo change to 'python' None
                });

                return ExecutionResult::Success {
                    flow: FlowControl::Continue,
                    dependencies: dependencies,
                    changes: changes,
                    results: values,
                }

            },
            _ => panic!("bad shit"),

        }
    }

    fn assign_to_target(&mut self, target: &GastNode, mappings: &Vec<Mapping>) -> ExecutionResult {
        match target {
            &GastNode::Identifier {ref name} => {
                self.assign_to_identifier(name, mappings)
            },
            // list
            // sequence
            // attribute
            _ => panic!("unimplemented"),
        }
    }

    fn assign_to_identifier(&mut self, target: &String, mappings: &Vec<Mapping>) -> ExecutionResult {
        match self.contexts.last_mut() {
            Some(mut last) => {
                let scope = last.get_public_scope_mut();
                scope.invalidate_mappings(target);
                for mapping in mappings {
                    scope.add_mapping(target, mapping.clone());
                }
            },
            _ => panic!("No Execution Contexts")
        }

        let values = vec!(Result {
            assumption: Assumption::None,
            value: -1, //todo change to 'python' None
        });

        return ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec!(),
            changes: vec!(Change::Identifier {name: target.clone()} ),
            results: values,
        }
    }


    fn create_simple_mapping(value: Pointer) -> Mapping {
        return Mapping { assumption: Assumption::None, address: value }
    }

    pub fn inspect_identifier(&self, name: &String) {
        let mut candidate = None;

        for context in self.contexts.iter().rev() {
            candidate = context.get_public_scope().resolve_identifier(&name);

            if candidate.is_some() {
                break;
            }
        }

        match candidate {
            Some(mappings) => {
                for ref mapping in mappings {
                    self.print_mapping_info(name, &mapping);
                }
            },
            None => panic!("resolving unknown identifier"),
        }
    }

    fn print_mapping_info(&self, name: &String, mapping: &Mapping) {
        match mapping {
            &Mapping {ref assumption, ref address} => {
                let object = self.memory.get_object(address);
                let tpe = object.get_extension();

                match tpe {
                    &Some(ref type_pointer) => {
                        let type_name = self.knowledge_base.get_type_name(type_pointer);
                        println!("Object {:?} has type {:?} in {:?}", name, type_name.unwrap(), mapping);
                    },
                    _ => println!("{:?} is a type in {:?}", name, mapping),
                }
            }
        }

    }

    pub fn declare_simple_type(&mut self, name: &String) {
        let pointer = self.memory.new_object();
        self.knowledge_base.add_type(name.clone(), pointer.clone());
        self.assign_to_identifier(name, &vec!(Mapping::new(Assumption::None, pointer)));
    }

    pub fn new_context(&mut self) {
        self.contexts.push(Context::new());
    }

}

// Giving the compiler something to do
fn main() {
    let mut vm = VirtualMachine::new();

    vm.new_context();

    vm.declare_simple_type(&"number".to_owned());

    let x = GastNode::Identifier { name: "x".to_owned(), };
    let value = Box::new(GastNode::Number { value: 5, });
    let assignment = GastNode::Assignment{
        targets: vec!(x),
        value: value,
    };
    let result = vm.execute(&assignment);

    println!("{:?}", result);

    vm.inspect_identifier(&"number".to_owned());
    vm.inspect_identifier(&"x".to_owned());
}
