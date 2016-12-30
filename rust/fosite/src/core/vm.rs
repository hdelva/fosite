use super::*;

use std::collections::HashSet;
use carboxyl::Sink;

pub struct VirtualMachine {
    // instruction queue
    // call stack
    contexts: Vec<Context>,
    pub memory: Memory, // todo make private
    knowledge_base: KnowledgeBase,

    assumptions: Vec<Assumption>,
    nodes: Vec<GastID>,
}

impl VirtualMachine {
    pub fn new() -> VirtualMachine {
        let memory = Memory::new();
        let knowledge = KnowledgeBase::new();
        VirtualMachine {
            contexts: Vec::new(),
            memory: memory,
            knowledge_base: knowledge,
			nodes: vec!(),
            assumptions: vec![Assumption::empty()],
        }
    }

    pub fn execute(&mut self, node: &GastNode) -> ExecutionResult {
        let ref id = node.id;
        let ref kind = node.kind;
        
        self.nodes.push(id.clone());

        let result = match kind {
            &NodeType::Number { .. } => self.number(),
            &NodeType::Identifier { ref name } => self.load_identifier(name),
            &NodeType::String { .. } => self.string(),
            &NodeType::Declaration { ref id, ref kind } => self.declaration(id, kind),
            &NodeType::Assignment { ref targets, ref value } => self.assign(targets, value),
            &NodeType::Block { ref content } => self.block(content),
            &NodeType::Attribute { ref parent, ref attribute } => self.load_attribute(id, parent, attribute),
            &NodeType::If { ref test, ref body, ref or_else } => self.conditional(test, body, or_else),
            _ => panic!("Unsupported Operation"),
        };

        let message = Message::Notification {
            source: id.clone(),
            content: format!("{:?}", result),
        };
        
        let _ = self.nodes.pop();
        
        &CHANNEL.publish(message);
        return result;
    }
    
    fn conditional(&mut self, test: &GastNode, body: &GastNode, or_else: &GastNode) -> ExecutionResult {
    	let _ = self.execute(test);
    	
    	let last_assumption = self.assumptions.pop().unwrap();
    	
    	let mut changed_objects = HashSet::new();
    	
    	let mut positive_assumption = last_assumption.clone();
    	positive_assumption.add(self.nodes.last().unwrap().clone(), false);
    	let mut negative_assumption = last_assumption.clone();
    	negative_assumption.add(self.nodes.last().unwrap().clone(), true);
    	
    	self.assumptions.push(positive_assumption);
    	let body_result = self.execute(body);
    	let _ = self.assumptions.pop();
    	
    	match body_result {
    		ExecutionResult::Success {ref changes, ..} => {
    			for change in changes {
    				if let &AnalysisItem::Object {address} = change {
    					changed_objects.insert(address);
    				}
    			}
    		},
    		_ => ()
    	}
    	
    	for address in &changed_objects {
    		let mut object = self.memory.get_object_mut(address);
    		object.change_branch();
    	}
    	
    	self.assumptions.push(negative_assumption);
    	let else_result = self.execute(or_else);
    	let _ = self.assumptions.pop();
    	
    	match else_result {
    		ExecutionResult::Success {ref changes, ..} => {
    			for change in changes {
    				if let &AnalysisItem::Object {address} = change {
    					changed_objects.insert(address);
    				}
    			}
    		},
    		_ => ()
    	}
    	
    	for address in &changed_objects {
    		let mut object = self.memory.get_object_mut(address);
    		object.merge_branches();
    	}
    	
    	self.assumptions.push(last_assumption);
    	
    	return body_result;
    }
    
    fn block(&mut self, content: &Vec<GastNode>) -> ExecutionResult {
    	let mut total_dependencies = Vec::new();
    	let mut total_changes = Vec::new();
    	
    	for node in content {
    		let mut intermediate = self.execute(node);
    		
    		match intermediate {
    			ExecutionResult::Success {ref mut dependencies, ref mut changes, ..} => {
    				total_dependencies.append(dependencies);
    				total_changes.append(changes);
    			},
    			_ => panic!("executing block went wrong")
    		}
    	}
    	
    	return ExecutionResult::Success {
    		flow: FlowControl::Continue,
    		dependencies: total_dependencies,
    		changes: total_changes,
    		result: Mapping::new(),
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
            }
            _ => panic!("system isn't properly initialized"),
        }

        let mapping = Mapping::simple(Assumption::empty(), pointer.clone());

        let execution_result = ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![],
            result: mapping,
        };

        return execution_result;
    }

    // todo, add assumption merging
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
                let mut mapping = Mapping::new();
                mapping.add_mapping(Assumption::empty(), pointer.clone());
                scope.set_mapping(name.clone(),
                                  self.assumptions.last().unwrap().clone(),
                                  mapping);
            }
            _ => panic!("No Execution Contexts"),
        }

        let mapping = Mapping::simple(Assumption::empty(), -1); // todo change to python None

        let execution_result = ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![AnalysisItem::Identifier { name: name.clone() }],
            result: mapping,
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

        let mapping = Mapping::simple(Assumption::empty(), pointer.clone());

        let execution_result = ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![],
            result: mapping,
        };

        return execution_result;
    }

    // todo, add LEGB
    fn load_identifier(&mut self, name: &String) -> ExecutionResult {
        let context = self.contexts.last().unwrap();
        
        //todo, change resolve_optional_identifier
        let mapping = context.get_public_scope().resolve_identifier(&name);

        let execution_result = ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec![AnalysisItem::Identifier {name: name.clone()} ],
            changes: vec![],
            result: mapping,
        };

        return execution_result;

    }

	fn load_object_attribute(&self, address: &Pointer, name: &String) -> OptionalMapping {
		let mut unresolved = Vec::new();
		
		let object = self.memory.get_object(address);
		let opt_mappings = object.get_attribute(name);
		
		let mut result = OptionalMapping::new();		
		
		for (ass, opt_address) in opt_mappings.iter() {            	
            if let &Some(address) = opt_address {
            	result.add_mapping(ass.clone(), Some(address.clone()));
            } else {
            	unresolved.push(ass.clone());
            }
        }
		
		if unresolved.len() > 0 {
        	let types = object.get_extension();
        	
        	if types.len() == 0 {
        		// can't go further up the hierarchy
        		result.add_mapping(Assumption::empty(), None);
        	}
        	
    		for tpe in types {
    			let mut found = true;
    			
    			for (ass, opt_address) in self.load_object_attribute(tpe, name).into_iter() {
    				if opt_address.is_none() {
    					found = false;
    				}
    				
    				for original in unresolved.iter() {
    					let mut new_ass = ass.clone();
    					for pls in original.iter() {
    						new_ass.add_element(pls.clone());
    					}
    					result.add_mapping(new_ass, opt_address.clone());
    				}
    			}
    			
    			if found {
    				//todo, technically we should adjust the unresolved vector now
    				// the next type only gets explored if this one returned nothing
    				break;
    			}
        	}
        } 
		
		return result;
	}
    
    fn load_attribute(&mut self, source: &GastID, parent: &GastNode, name: &String) -> ExecutionResult {
        let mut parent_result = self.execute(parent);
        
        let mut total_dependencies = Vec::new();
        let mut mapping = Mapping::new();
        
        // which assumptions still need a valid mapping
        let mut unresolved = Vec::new();
        
        match parent_result {
            ExecutionResult::Success { ref result, ref mut dependencies, .. } => {
                let parent_mapping = result;
                
                for dependency in dependencies.iter() {
                	total_dependencies.push(AnalysisItem::Attribute { parent: Box::new(dependency.clone()), name: name.clone() });
                }
                
                total_dependencies.append(dependencies);
                
                for (parent_assumption, parent_address) in parent_mapping.iter() {
                	dependencies.push( AnalysisItem::Object { address: parent_address.clone() });
                	
                    let parent_object = self.memory.get_object(parent_address);
                    let opt_mappings = parent_object.get_attribute(name);
                                        
                    for (ass, opt_address) in opt_mappings.iter() {
                    	
	                    if let &Some(address) = opt_address {	                    	
	                    	let mut new_ass = ass.clone();
        					for pls in parent_assumption.iter() {
        						new_ass.add_element(pls.clone());
        					}
	                    	
	                    	mapping.add_mapping(new_ass, address.clone());
	                    } else {
	                    	unresolved.push(ass.clone());
	                    	
	                    	if opt_mappings.len() > 1 {
		                    	// having a single None is fine
		                    	// probably a class method then
		                    	
		                    	let message = Message::Warning {
		                    		source: source.clone(),
		                    		assumption: ass.clone(),
		                    		content: "object does not always have an attribute of this name".to_owned(),
		                    	};
		                    	&CHANNEL.publish(message);
		                    }
	                    }
                    }
                    
                    // look for the attribute in its types
                    if unresolved.len() > 0 {
			        	let types = parent_object.get_extension();
			        	
			        	if types.len() == 0 {
			        		for unmet in unresolved.iter() {
			        			//todo, add type information as well
			        			let message = Message::Error {
		                    		source: source.clone(),
		                    		assumption: unmet.clone(),
		                    		content: "object does not have an attribute of this name".to_owned(),
		                    	};
		                    	&CHANNEL.publish(message);
			        		}
			        		
	                    	continue;
			        	}
			        	
			        	for tpe in types.iter() {
			        		for (ass, opt_address) in self.load_object_attribute(tpe, name).into_iter() {
		        				for original in unresolved.iter() {
		        					let mut new_ass = ass.clone();
		        					for pls in original.iter() {
		        						new_ass.add_element(pls.clone());
		        					}
		        					
		        					if opt_address.is_none() {
					        			//todo, add type information as well
					        			let message = Message::Error {
				                    		source: source.clone(),
				                    		assumption: new_ass.clone(),
				                    		content: "object does not have an attribute of this name".to_owned(),
				                    	};
				                    	&CHANNEL.publish(message);
				                    	continue;
					        		} else {
			        					mapping.add_mapping(new_ass, opt_address.unwrap());
					        		}
		        				}
		        			}
			        	}
			        } 
                }
            }
            _ => panic!("invalid attribute parent"),
        }
        
        return ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: Vec::new(),
            result: mapping,
        };
    }


    fn assign(&mut self, targets: &Vec<GastNode>, value: &GastNode) -> ExecutionResult {
        let value_execution = self.execute(value);

        match value_execution {
            ExecutionResult::Success { dependencies, mut changes, result, .. } => {
                for target in targets {
                    let partial_result = self.assign_to_target(target, &result);

                    match partial_result {
                        ExecutionResult::Success { changes: mut t_changes, .. } => {
                            changes.append(&mut t_changes)
                        }
                        _ => panic!("bad shit"),
                    }
                }

                // todo change to python None
                let mapping = Mapping::simple(Assumption::empty(), -1);

                return ExecutionResult::Success {
                    flow: FlowControl::Continue,
                    dependencies: dependencies,
                    changes: changes,
                    result: mapping,
                };

            }
            _ => panic!("bad shit"),

        }
    }

    fn assign_to_target(&mut self, target: &GastNode, mapping: &Mapping) -> ExecutionResult {
        match &target.kind {
            &NodeType::Identifier { ref name } => self.assign_to_identifier(name, mapping),
            &NodeType::List { ref content } |
            &NodeType::Sequence { ref content } => self.assign_to_iterable(content, mapping),
            &NodeType::Attribute { ref parent, ref attribute } => {
                self.assign_to_attribute(parent, attribute, mapping)
            },
            // attribute
            _ => panic!("unimplemented"),
        }
    }

    fn assign_to_attribute(&mut self,
                           parent: &GastNode,
                           attribute: &String,
                           mapping: &Mapping)
                           -> ExecutionResult {

        let parent_result = self.execute(parent);
        
        match parent_result {
            ExecutionResult::Success { result, dependencies, .. } => {
                let parent_mapping = result;
                let mut changes = Vec::new();
                
                // add the attribute identifier changes
                for dependency in dependencies.into_iter() {
                	changes.push(AnalysisItem::Attribute { parent: Box::new(dependency), name: attribute.clone() });
                }

				// add the object changes
				// perform the assignment
                for (_, parent_address) in parent_mapping.iter() {
                	changes.push( AnalysisItem::Object { address: parent_address.clone() });
                	
                    let mut parent_object = self.memory.get_object_mut(parent_address);
                    parent_object.assign_attribute(attribute.clone(),
                                                   self.assumptions.last().unwrap().clone(),
                                                   mapping.clone())
                }

                return ExecutionResult::Success {
                    flow: FlowControl::Continue,
                    dependencies: vec![],
                    changes: changes,
                    result: Mapping::new(),
                };
            }
            _ => panic!("invalid attribute parent"),
        }
    }

    // todo rewrite
    fn assign_to_iterable(&mut self, target: &Vec<GastNode>, mapping: &Mapping) -> ExecutionResult {

        // for mapping in mappings {
        // let ref assumption = mapping.assumption;
        // let ref address = mapping.address;
        // let object = self.memory.get_object(&address);
        //
        // match object.iterate() {
        // Some(sub_address) => {
        // new_mappings.push(Mapping::new(assumption.clone(), sub_address));
        // }
        // _ => panic!("object isn't iterable"),
        // }
        // }
        //
        // let mut new_changes = Vec::new();
        //
        // for sub_target in target {
        // let sub_result = self.assign_to_target(sub_target, &new_mappings);
        //
        // match sub_result {
        // ExecutionResult::Success { mut changes, .. } => {
        // new_changes.append(&mut changes);
        // }
        // _ => (),
        // }
        // }

        return ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![],
            result: Mapping::new(),
        };
    }

    fn assign_to_identifier(&mut self, target: &String, mapping: &Mapping) -> ExecutionResult {
        match self.contexts.last_mut() {
            Some(mut last) => {
                let scope = last.get_public_scope_mut();
                scope.set_mapping(target.clone(),
                                  self.assumptions.last().unwrap().clone(),
                                  mapping.clone());
            }
            _ => panic!("No Execution Contexts"),
        }

        let mapping = Mapping::simple(Assumption::empty(), -1);

        return ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![AnalysisItem::Identifier { name: target.clone() }],
            result: mapping,
        };
    }

    pub fn inspect_identifier(&self, name: &String) {
        let context = self.contexts.last().unwrap();

        let candidate = context.get_public_scope().resolve_optional_identifier(&name);

        for (assumption, opt_address) in candidate.iter() {
            self.print_mapping_info(name, assumption, opt_address)
        }
    }

    fn print_mapping_info(&self, name: &String, ass: &Assumption, address: &Option<Pointer>) {
        let object = self.memory.get_object(&address.unwrap());
        let tpe = object.get_extension().first();

        match tpe {
            Some(ref type_pointer) => {
                let type_name = self.knowledge_base.get_type_name(type_pointer);
                println!("Assuming {:?}, the Object named {:?} at address {:?} has type {:?}",
                         ass,
                         name,
                         address.unwrap(),
                         type_name.unwrap());
            }
            _ => {
                if object.is_type() {
                    println!("Assuming {:?}, {:?} is a type in defined at {:?}",
                             ass,
                             name,
                             address.unwrap())
                } else {
                    println!("Assuming {:?}, {:?} is an object of unknown type at address {:?}",
                             ass,
                             name,
                             address.unwrap())
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
        self.assign_to_identifier(name, &Mapping::simple(Assumption::empty(), pointer));
    }

    pub fn new_context(&mut self) {
        self.contexts.push(Context::new());
    }

    // fn resolve_attribute(&mut self, parent: &Pointer, name: &String) -> ExecutionResult {
    // let object = self.memory.get_object(parent);
    // let locals = object.get_attribute(name);
    //
    //
    // }
    //
}
