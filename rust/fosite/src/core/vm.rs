use super::*;

use std::collections::HashSet;
use carboxyl::Sink;

pub struct VirtualMachine {
    // instruction queue
    // call stack
    contexts: Vec<Context>,
    pub memory: Memory, //todo make private
    knowledge_base: KnowledgeBase,
    stream: Sink<Message>,
}

impl VirtualMachine {
    pub fn new(stream: Sink<Message>) -> VirtualMachine {
        let memory = Memory::new();
        let knowledge = KnowledgeBase::new();
        VirtualMachine {
            contexts: Vec::new(),
            memory: memory,
            knowledge_base: knowledge,
            stream: stream,
        }
    }

    pub fn execute(&mut self, node: &GastNode) -> ExecutionResult {
        let ref id = node.id;
        let ref kind = node.kind;

        let result = match kind {
            &NodeType::Number { .. } => self.number(),
            &NodeType::Identifier { ref name } => self.load_identifier(name),
            &NodeType::String { .. } => self.string(),
            &NodeType::Declaration { ref id, ref kind } => self.declaration(id, kind),
            &NodeType::Assignment { ref targets, ref value } => self.assign(targets, value),
            _ => panic!("Unsupported Operation"),
        };

        let message = Message::Notification{source: id.clone(), content: format!("{:?}", result)};
        self.stream.send(message);
        return result;
    }

    fn string(&mut self) -> ExecutionResult {
        let pointer = self.memory.new_object();
        let object = self.memory.get_object_mut(&pointer);
        let type_name = "string".to_owned();
        let type_pointer = self.knowledge_base.get_type(&type_name);

        match type_pointer {
            Some(address) => {
                object.extends(address.clone());
            }
            _ => panic!("system isn't properly initialized"),
        }

        let result = Result {
            assumption: Assumption::None,
            value: pointer.clone(),
        };

        let execution_result = ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![],
            results: vec![result],
        };

        return execution_result;
    }

    fn declaration(&mut self, name: &String, type_name: &String) -> ExecutionResult {
        let pointer = self.memory.new_object();
        let object = self.memory.get_object_mut(&pointer);
        let type_pointer = self.knowledge_base.get_type(type_name);

        match type_pointer {
            Some(address) => {
                object.extends(address.clone());
            }
            _ => panic!("declaration type does not exist"),
        }

        let mut possibilities = HashSet::new();
        possibilities.insert(pointer.clone());

        match self.contexts.last_mut() {
            Some(mut last) => {
                let scope = last.get_public_scope_mut();
                scope.invalidate_mappings(name);
                let mapping = VirtualMachine::create_simple_mapping(pointer);
                scope.add_mapping(name, mapping);
            }
            _ => panic!("No Execution Contexts"),
        }

        let result = Result {
            assumption: Assumption::None,
            value: -1, // todo change to 'python' None
        };

        let execution_result = ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![Change::Identifier { name: name.clone() }],
            results: vec![result],
        };

        return execution_result;
    }

    fn number(&mut self) -> ExecutionResult {
        let pointer = self.memory.new_object();
        let object = self.memory.get_object_mut(&pointer);
        let type_name = "number".to_owned();
        let type_pointer = self.knowledge_base.get_type(&type_name);

        match type_pointer {
            Some(address) => {
                object.extends(address.clone());
            }
            _ => panic!("system isn't properly initialized"),
        }

        let result = Result {
            assumption: Assumption::None,
            value: pointer.clone(),
        };

        let execution_result = ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![],
            results: vec![result],
        };

        return execution_result;
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

                    results.push(Result {
                        assumption: assumption,
                        value: address,
                    })
                }

                let execution_result = ExecutionResult::Success {
                    flow: FlowControl::Continue,
                    dependencies: vec![name.clone()],
                    changes: vec![],
                    results: results,
                };

                return execution_result;
            }
            _ => panic!("Invalid Identifier"),
        }
    }


    fn assign(&mut self, targets: &Vec<GastNode>, value: &GastNode) -> ExecutionResult {
        let value_execution = self.execute(value);

        match value_execution {
            ExecutionResult::Success { dependencies, mut changes, results, .. } => {
                let mut mappings = Vec::new();

                for result in results {
                    mappings.push(Mapping {
                        assumption: result.assumption,
                        address: result.value,
                    })
                }

                for target in targets {
                    let partial_result = self.assign_to_target(target, &mappings);

                    match partial_result {
                        ExecutionResult::Success { changes: mut t_changes, .. } => {
                            changes.append(&mut t_changes)
                        }
                        _ => panic!("bad shit"),
                    }
                }

                let values = vec![Result {
                    assumption: Assumption::None,
                    value: -1, // todo change to 'python' None
                }];

                return ExecutionResult::Success {
                    flow: FlowControl::Continue,
                    dependencies: dependencies,
                    changes: changes,
                    results: values,
                };

            }
            _ => panic!("bad shit"),

        }
    }

    fn assign_to_attribute(&mut self,
                           parent: &GastNode,
                           attribute: &String,
                           mappings: &Vec<Mapping>)
                           -> ExecutionResult {
        let parent_result = self.execute(parent);
        match parent_result {
            ExecutionResult::Success { results, .. } => {
                let mut changes = Vec::new();
                // todo how to model attribute changes
                changes.push(Change::Identifier { name: attribute.clone() });

                for parent_mapping in results {
                    let ref parent_assumption = parent_mapping.assumption;
                    let address = parent_mapping.value;
                    changes.push(Change::Object { address: address });

                    let mut object = self.memory.get_object_mut(&address);

                    for value_mapping in mappings {
                        let ref value_assumption = value_mapping.assumption;
                        let ref value_address = value_mapping.address;

                        let new_assumption = VirtualMachine::merge_assumptions(parent_assumption,
                                                                               value_assumption);
                        let new_mapping = Mapping::new(new_assumption, value_address.clone());

                        object.assign_attribute(attribute.clone(), new_mapping);
                    }
                }

                return ExecutionResult::Success {
                    flow: FlowControl::Continue,
                    dependencies: vec![],
                    changes: changes,
                    results: vec![],
                };
            }
            _ => panic!("invalid attribute parent"),
        }
    }

    fn merge_assumptions<'a>(ass1: &'a Assumption, ass2: &'a Assumption) -> Assumption {
        match (ass1, ass2) {
            (&Assumption::None, other) |
            (other, &Assumption::None) => other.clone(),
            (&Assumption::Multiple(ref assumptions),
                &Assumption::ConditionAssumption { source, negated }) |
            (&Assumption::ConditionAssumption { source, negated },
                &Assumption::Multiple(ref assumptions)) => {
                let mut new_assumptions = assumptions.clone();
                new_assumptions.push(Assumption::ConditionAssumption {
                    source: source,
                    negated: negated,
                });
                return Assumption::Multiple(new_assumptions);
            }
            (&Assumption::Multiple(ref first), &Assumption::Multiple(ref second)) => {
                let mut new_assumptions = first.clone();
                let mut pls = second.clone();
                new_assumptions.append(&mut pls);
                return Assumption::Multiple(new_assumptions);
            }
            (first, second) => {
                return Assumption::Multiple(vec![first.clone(), second.clone()]);
            }

        }
    }

    fn assign_to_target(&mut self, target: &GastNode, mappings: &Vec<Mapping>) -> ExecutionResult {
        match &target.kind {
            &NodeType::Identifier { ref name } => self.assign_to_identifier(name, mappings),
            &NodeType::List { ref content } |
            &NodeType::Sequence { ref content } => self.assign_to_iterable(content, mappings),
            &NodeType::Attribute { ref parent, ref attribute } => {
                self.assign_to_attribute(parent, attribute, mappings)
            }
            // attribute
            _ => panic!("unimplemented"),
        }
    }

    fn assign_to_iterable(&mut self,
                          target: &Vec<GastNode>,
                          mappings: &Vec<Mapping>)
                          -> ExecutionResult {
        let mut new_mappings = Vec::new();
        for mapping in mappings {
            let ref assumption = mapping.assumption;
            let ref address = mapping.address;
            let object = self.memory.get_object(&address);

            match object.iterate() {
                Some(sub_address) => {
                    new_mappings.push(Mapping::new(assumption.clone(), sub_address));
                }
                _ => panic!("object isn't iterable"),
            }
        }

        let mut new_changes = Vec::new();

        for sub_target in target {
            let sub_result = self.assign_to_target(sub_target, &new_mappings);

            match sub_result {
                ExecutionResult::Success { mut changes, .. } => {
                    new_changes.append(&mut changes);
                }
                _ => (),
            }
        }

        return ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: new_changes,
            results: vec![],
        };
    }

    fn assign_to_identifier(&mut self,
                            target: &String,
                            mappings: &Vec<Mapping>)
                            -> ExecutionResult {
        match self.contexts.last_mut() {
            Some(mut last) => {
                let scope = last.get_public_scope_mut();
                scope.invalidate_mappings(target);
                for mapping in mappings {
                    scope.add_mapping(target, mapping.clone());
                }
            }
            _ => panic!("No Execution Contexts"),
        }

        let values = vec![Result {
            assumption: Assumption::None,
            value: -1, // todo change to 'python' None
        }];

        return ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![Change::Identifier { name: target.clone() }],
            results: values,
        };
    }

    fn create_simple_mapping(value: Pointer) -> Mapping {
        return Mapping {
            assumption: Assumption::None,
            address: value,
        };
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
            }
            None => panic!("resolving unknown identifier"),
        }
    }

    fn print_mapping_info(&self, name: &String, mapping: &Mapping) {
        match mapping {
            &Mapping { ref address, .. } => {
                let object = self.memory.get_object(address);
                let tpe = object.get_extension();

                match tpe {
                    &Some(ref type_pointer) => {
                        let type_name = self.knowledge_base.get_type_name(type_pointer);
                        println!("Object {:?} has type {:?} in {:?}",
                                 name,
                                 type_name.unwrap(),
                                 mapping);
                    }
                    _ => {
                        if object.is_type() {
                            println!("{:?} is a type in {:?}", name, mapping)
                        } else {
                            println!("{:?} is an object of unknown type in {:?}", name, mapping)
                        }
                    }
                }
            }
        }

    }

    pub fn declare_simple_type(&mut self, name: &String) {
        let pointer = self.memory.new_object();
        {
            let mut object = self.memory.get_object_mut(&pointer);
            object.set_type(true);
        }
        self.knowledge_base.add_type(name.clone(), pointer.clone());
        self.assign_to_identifier(name, &vec![Mapping::new(Assumption::None, pointer)]);
    }

    pub fn new_context(&mut self) {
        self.contexts.push(Context::new());
    }
}