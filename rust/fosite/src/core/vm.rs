use super::*;

use std::collections::HashSet;
use std::collections::HashMap;

pub struct VirtualMachine {
    // instruction queue
    // call stack
    scopes: Vec<Scope>,
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
            scopes: Vec::new(),
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
            &NodeType::Int { .. } => self.int(),
            &NodeType::Float { .. } => self.float(),
            &NodeType::Identifier { ref name } => self.load_identifier(name),
            &NodeType::String { .. } => self.string(),
            &NodeType::Declaration { ref id, ref kind } => self.declaration(id, kind),
            &NodeType::Assignment { ref targets, ref value } => self.assign(targets, value),
            &NodeType::Block { ref content } => self.block(content),
            &NodeType::Attribute { ref parent, ref attribute } => self.load_attribute(id, parent, attribute),
            &NodeType::If { ref test, ref body, ref or_else } => self.conditional(test, body, or_else),
            _ => panic!("Unsupported Operation"),
        };

		{
			let mut items = HashMap::new();
			items.insert("node".to_owned(), MessageItem::String(format!("{:?}", result)));
	        let message = Message::Notification {
	            source: id.clone(),
	            kind: NPROCESSED_NODE,
	            content: items,
	        };
	        &CHANNEL.publish(message);
		}   
		    
        let _ = self.nodes.pop();
        
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
    	
        let mut scope_changes = false;

    	match body_result {
    		ExecutionResult::Success {ref changes, ..} => {
    			for change in changes {
    				if let &AnalysisItem::Object {address} = change {
    					changed_objects.insert(address);
    				} else if let &AnalysisItem::Identifier {..} = change {
                        scope_changes = true;
                    }
    			}
    		},
    		_ => ()
    	}
    	
    	for address in &changed_objects {
    		let mut object = self.memory.get_object_mut(address);
    		object.change_branch();
    	}

        if scope_changes {
            self.scopes.last_mut().unwrap().change_branch();
        }
    	
    	self.assumptions.push(negative_assumption);
    	let else_result = self.execute(or_else);
    	let _ = self.assumptions.pop();
    	
    	match else_result {
    		ExecutionResult::Success {ref changes, ..} => {
    			for change in changes {
    				if let &AnalysisItem::Object {address} = change {
    					changed_objects.insert(address);
    				} else if let &AnalysisItem::Identifier {..} = change {
                        scope_changes = true;
                    }
    			}
    		},
    		_ => ()
    	}
    	
    	for address in &changed_objects {
    		let mut object = self.memory.get_object_mut(address);
    		object.merge_branches();
    	}

        if scope_changes {
            self.scopes.last_mut().unwrap().merge_branches();
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

        let mut scope = self.scopes.last_mut().unwrap();

        let mut mapping = Mapping::new();
        mapping.add_mapping(Assumption::empty(), pointer.clone());
        scope.set_mapping(name.clone(),
                            self.assumptions.last().unwrap().clone(),
                            mapping);


        let mapping = Mapping::simple(Assumption::empty(), -1); // todo change to python None

        let execution_result = ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![AnalysisItem::Identifier { name: name.clone() }],
            result: mapping,
        };

        return execution_result;
    }

    fn int(&mut self) -> ExecutionResult {
        let pointer = self.memory.new_object();
        let object = self.memory.get_object_mut(&pointer);
        let type_name = "int".to_owned();
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

    fn float(&mut self) -> ExecutionResult {
        let pointer = self.memory.new_object();
        let object = self.memory.get_object_mut(&pointer);
        let type_name = "float".to_owned();
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
        let mut unresolved = vec!(Assumption::empty());

        let mut mapping = Mapping::new();

        for scope in self.scopes.iter().rev() {
            let opt_mappings = scope.resolve_optional_identifier(&name);

            let mut new_unresolved = Vec::new();

            for (ass, opt_address) in opt_mappings.iter() {   
                for unresolved_ass in &unresolved {
                    let mut new_ass = ass.clone();
                    for pls in unresolved_ass.iter() {
                        new_ass.add_element(pls.clone());
                    }

                    if let &Some(address) = opt_address {
                        mapping.add_mapping(new_ass, address.clone());
                    } else {
                        new_unresolved.push(new_ass.clone());

                        if opt_mappings.len() > 1 {
                            let mut items = HashMap::new();
                            items.insert("assumption".to_owned(), MessageItem::Assumption(new_ass));
                            
                            let message = Message::Warning {
                                source: self.nodes.last().unwrap().clone(),
                                kind: WIDENTIFIER_UNSAFE,
                                content: items,
                            };
                            &CHANNEL.publish(message);
                        }
                    }
                }         	
            }

            unresolved = new_unresolved;
            if unresolved.len() == 0 {
                break;
            }
        }


        if unresolved.len() > 0 {
            for unresolved_ass in unresolved {
                let mut items = HashMap::new();
                items.insert("assumption".to_owned(), MessageItem::Assumption(unresolved_ass.clone()));
                
                let message = Message::Error {
                    source: self.nodes.last().unwrap().clone(),
                    kind: EIDENTIFIER_INVALID,
                    content: items,
                };
                &CHANNEL.publish(message);
            }
        }

        

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
		                    	let mut items = HashMap::new();
		                    	items.insert("assumption".to_owned(), MessageItem::Assumption(ass.clone()));
		                    	
		                    	let message = Message::Warning {
		                    		source: source.clone(),
		                    		kind: WATTRIBUTE_UNSAFE,
		                    		content: items,
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
			        			let mut items = HashMap::new();
		                    	items.insert("assumption".to_owned(), MessageItem::Assumption(unmet.clone()));
		                    	
		                    	let message = Message::Error {
		                    		source: source.clone(),
		                    		kind: EATTRIBUTE_INVALID,
		                    		content: items,
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
					        			let mut items = HashMap::new();
				                    	items.insert("assumption".to_owned(), MessageItem::Assumption(new_ass.clone()));
				                    	
				                    	let message = Message::Error {
				                    		source: source.clone(),
				                    		kind: EATTRIBUTE_INVALID,
				                    		content: items,
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
        let mut scope = self.scopes.last_mut().unwrap();

        scope.set_mapping(target.clone(),
                            self.assumptions.last().unwrap().clone(),
                            mapping.clone());

        let mapping = Mapping::simple(Assumption::empty(), -1);

        return ExecutionResult::Success {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![AnalysisItem::Identifier { name: target.clone() }],
            result: mapping,
        };
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

    pub fn new_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    // fn resolve_attribute(&mut self, parent: &Pointer, name: &String) -> ExecutionResult {
    // let object = self.memory.get_object(parent);
    // let locals = object.get_attribute(name);
    //
    //
    // }
    //
}
