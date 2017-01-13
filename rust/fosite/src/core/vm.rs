use super::*;

use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::BTreeSet;
use std::iter::FromIterator;
use std::collections::hash_map::Entry;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

const NONE: Pointer = 1;
const TRUE: Pointer = 5;
const FALSE: Pointer = 6;

pub struct VirtualMachine {
    //todo call stack
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
            &NodeType::Attribute { ref parent, ref attribute } => self.load_attribute(parent, attribute),
            &NodeType::If { ref test, ref body, ref or_else } => self.conditional(test, body, or_else),
            &NodeType::BinOp {ref left, ref right, .. } => self.binop(left, right),
            &NodeType::Nil {} => self.load_identifier(&"None".to_owned()),
            &NodeType::Boolean { ref value } => self.boolean(*value),            
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

    fn boolean(&self, value: bool) -> ExecutionResult {
        if value {
            self.load_identifier(&"True".to_owned())
        } else {
            self.load_identifier(&"False".to_owned())
        }
    }

    fn binop(&mut self, left: &GastNode, right: &GastNode) -> ExecutionResult {
        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();

        let mut left_result = self.execute(left);
        let mut left_mapping = left_result.result;
        total_changes.append(&mut left_result.changes);
        total_dependencies.append(&mut left_result.dependencies);

        let mut right_result = self.execute(left);
        let mut right_mapping = right_result.result;
        total_changes.append(&mut right_result.changes);
        total_dependencies.append(&mut right_result.dependencies);

        for (left_ass, left_address) in left_mapping.iter() {

        }

        let mapping = Mapping::simple(Assumption::empty(), -1);

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: total_changes,
            result: mapping,
        };

        return execution_result;
    }
    
    fn conditional(&mut self, test: &GastNode, body: &GastNode, or_else: &GastNode) -> ExecutionResult {
        //todo add this

    	let _ = self.execute(test);
    	
    	let last_assumption = self.assumptions.pop().unwrap();
    	
    	let mut total_changes = HashSet::new();
        let mut total_dependencies = HashSet::new();
    	
    	let mut positive_assumption = last_assumption.clone();
    	positive_assumption.add(self.nodes.last().unwrap().clone(), true);
    	let mut negative_assumption = last_assumption.clone();
    	negative_assumption.add(self.nodes.last().unwrap().clone(), false);
    	
    	self.assumptions.push(positive_assumption);
    	let body_result = self.execute(body);
    	let _ = self.assumptions.pop();

        let mut identifier_changed = false;

        let changes = body_result.changes;
        let dependencies = body_result.dependencies;
    	
        for change in &changes {
            total_changes.insert(change.clone());
            
            if let &AnalysisItem::Identifier {..} = change {
                identifier_changed = true;
            }
        }

        for dependency in &dependencies {
            total_dependencies.insert(dependency.clone());
        }
    	
    	for change in &total_changes {
            if let &AnalysisItem::Object {ref address} = change {
                let mut object = self.memory.get_object_mut(address);
    		    object.change_branch();
            }
    	}

        if identifier_changed {
            self.scopes.last_mut().unwrap().change_branch();
        }
    	
    	self.assumptions.push(negative_assumption);
    	let else_result = self.execute(or_else);
    	let _ = self.assumptions.pop();
    	
        let changes = else_result.changes;
        let dependencies = else_result.dependencies;

        for change in &changes {
            total_changes.insert(change.clone());
            
            if let &AnalysisItem::Identifier {..} = change {
                identifier_changed = true;
            }
        }

        for dependency in &dependencies {
            total_dependencies.insert(dependency.clone());
        }

        self.assumptions.push(last_assumption);

        self.merge_branches(identifier_changed, &total_changes);

        self.check_conditional(&total_changes);
    	
        //todo make this sensible
    	return ExecutionResult {
            changes: Vec::from_iter(total_changes.into_iter()),
            dependencies: Vec::from_iter(total_dependencies.into_iter()),
            flow: FlowControl::Continue,
            result: Mapping::new(), //todo change to python None
        }
    }

    fn merge_branches(&mut self, identifier_changed: bool, changes: &HashSet<AnalysisItem> ) {
        if identifier_changed {
            self.scopes.last_mut().unwrap().merge_branches();
        }

    	for change in changes {
            if let &AnalysisItem::Object {ref address} = change {
                let mut object = self.memory.get_object_mut(address);
    		    object.merge_branches();
            }
    	}
    }

    // has to be mutable because there are executions inside
    fn check_conditional(&mut self, changes: &HashSet<AnalysisItem>) {
        for change in changes {
            if !change.is_object() {
                let mut all_types = HashMap::new();
                
                let execution_result = match change {
                    &AnalysisItem::Identifier {ref name} => self.load_identifier(name),
                    &AnalysisItem::Attribute {ref parent, ref name} => self.load_attribute(&parent.as_node(), name),
                    _ => panic!("AnalysisItem is an object when a previous check should've excluded this"),
                };

                

                let result = execution_result.result;
                for (assumption, address) in result.iter() {
                    let object = self.memory.get_object(address);
                    let tpe = object.get_extension()[0];

                    match all_types.entry(tpe.clone()) {
                        Entry::Vacant(v) => {
                            v.insert(vec!(assumption.clone()));
                        },
                        Entry::Occupied(mut o) => {
                            o.get_mut().push(assumption.clone());
                        },
                    };
                }

                if all_types.len() > 1 {
                    let mut items = HashMap::new();

                    items.insert("name".to_owned(), MessageItem::String(change.to_string()));

                    let mut type_count = 0;
                    for (tpe, assumptions) in all_types {
                        let type_name = self.knowledge_base.get_type_name(&tpe).unwrap();
                        items.insert(format!("type {}", type_count), MessageItem::String(type_name.clone()));

                        let mut ass_count = 0;
                        for assumption in assumptions {
                            items.insert(format!("type {} assumption {}", type_count, ass_count), MessageItem::Assumption(assumption.clone()));
                            ass_count += 1;
                        }
                        type_count += 1;
                    }

                    let kind = if change.is_identifier() {
                            WIDENTIFIER_POLY_TYPE
                        } else {
                            WATTRIBUTE_POLY_TYPE
                        };

                    let message = Message::Warning {
                        source: self.nodes.last().unwrap().clone(),
                        kind: kind,
                        content: items,
                    };

                    &CHANNEL.publish(message);
                }
            }
        }
    }
    
    fn block(&mut self, content: &Vec<GastNode>) -> ExecutionResult {
    	let mut total_dependencies = Vec::new();
    	let mut total_changes = Vec::new();
    	
    	for node in content {
    		let intermediate = self.execute(node);

            let mut dependencies = intermediate.dependencies;
            let mut changes = intermediate.changes;
    		
            total_dependencies.append(&mut dependencies);
            total_changes.append(&mut changes);
    	}
    	
    	return ExecutionResult {
    		flow: FlowControl::Continue,
    		dependencies: total_dependencies,
    		changes: total_changes,
    		result: Mapping::new(),
    	}
    } 

    fn object_of_type(&mut self, type_name: &String) -> Pointer {
        let pointer = self.memory.new_object();
        let object = self.memory.get_object_mut(&pointer);
        let type_pointer = self.knowledge_base.get_type(&type_name);

        match type_pointer {
            Some(address) => {
                object.extend(address.clone());
            }
            _ => panic!("There is no type with name {}", type_name),
        }

        return pointer;
    }

    fn string(&mut self) -> ExecutionResult {
        let type_name = "string".to_owned();
        let pointer = self.object_of_type(&type_name);

        let mapping = Mapping::simple(Assumption::empty(), pointer.clone());

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![],
            result: mapping,
        };

        return execution_result;
    }

    fn declaration(&mut self, name: &String, type_name: &String) -> ExecutionResult {
        let pointer = self.object_of_type(type_name);

        let mut possibilities = HashSet::new();
        possibilities.insert(pointer.clone());

        let mut scope = self.scopes.last_mut().unwrap();

        let mut mapping = Mapping::new();
        mapping.add_mapping(Assumption::empty(), pointer.clone());
        scope.set_mapping(name.clone(),
                            self.assumptions.last().unwrap().clone(),
                            mapping);


        let mapping = Mapping::simple(Assumption::empty(), -1); // todo change to python None

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![AnalysisItem::Identifier { name: name.clone() }],
            result: mapping,
        };

        return execution_result;
    }

    fn int(&mut self) -> ExecutionResult {
        let type_name = "int".to_owned();
        let pointer = self.object_of_type(&type_name);

        let mapping = Mapping::simple(Assumption::empty(), pointer.clone());

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![],
            result: mapping,
        };

        return execution_result;
    }

    fn float(&mut self) -> ExecutionResult {
        let type_name = "float".to_owned();
        let pointer = self.object_of_type(&type_name);

        let mapping = Mapping::simple(Assumption::empty(), pointer.clone());

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![],
            result: mapping,
        };

        return execution_result;
    }

    fn load_identifier(&self, name: &String) -> ExecutionResult {
        let mut unresolved = vec!(Assumption::empty());

        let mut mapping = Mapping::new();

        let mut warning = Vec::new();

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
                            warning.push(new_ass);
                        }
                    }
                }         	
            }

            unresolved = new_unresolved;
            if unresolved.len() == 0 {
                break;
            }
        }

        if warning.len() > 0 {
            let mut items = HashMap::new();

            items.insert("name".to_owned(), MessageItem::String(name.clone()));

            let mut ass_count = 0;
            for assumption in warning {
                items.insert(format!("assumption {}", ass_count), MessageItem::Assumption(assumption.clone()));
                ass_count += 1;
            }
            
            let message = Message::Warning {
                source: self.nodes.last().unwrap().clone(),
                kind: WIDENTIFIER_UNSAFE,
                content: items,
            };
            &CHANNEL.publish(message);
        }

        if unresolved.len() > 0 {
            let mut items = HashMap::new();

            items.insert("name".to_owned(), MessageItem::String(name.clone()));

            let mut ass_count = 0;
            for assumption in unresolved {
                items.insert(format!("assumption {}", ass_count), MessageItem::Assumption(assumption.clone()));
                ass_count += 1;
            }
            
            let message = Message::Error {
                source: self.nodes.last().unwrap().clone(),
                kind: EIDENTIFIER_INVALID,
                content: items,
            };
            &CHANNEL.publish(message);
        }

        let execution_result = ExecutionResult {
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
    
    fn load_attribute(&mut self, parent: &GastNode, name: &String) -> ExecutionResult {
        let parent_result = self.execute(parent);
        
        let mut total_dependencies = Vec::new();
        let mut total_changes = Vec::new();
        let mut mapping = Mapping::new();
        
        // which assumptions still need a valid mapping
        let mut unresolved = BTreeSet::new();

        let parent_mapping = parent_result.result;
        let mut dependencies = parent_result.dependencies;
        let mut changes = parent_result.changes;
                
        for dependency in dependencies.iter() {
            total_dependencies.push(AnalysisItem::Attribute { parent: Box::new(dependency.clone()), name: name.clone() });
        }
        
        total_dependencies.append(&mut dependencies);
        total_changes.append(&mut changes);

        let mut warning = BTreeSet::new();
        let mut error = BTreeSet::new();

        for (parent_assumption, parent_address) in parent_mapping.iter() {
            dependencies.push( AnalysisItem::Object { address: parent_address.clone() });
            
            let parent_object = self.memory.get_object(parent_address);
            let opt_mappings = parent_object.get_attribute(name);
                                
            'outer: for (ass, opt_address) in opt_mappings.iter() {
                let mut new_ass = parent_assumption.clone();
                for new_element in ass.iter() {
                    // avoid duplicate and conflicting assumptions
                    if new_ass.contains(new_element) {
                        continue;
                    } else if new_ass.contains_complement(new_element) {
                        continue 'outer;
                    }

                    new_ass.add_element(new_element.clone());
                }
                
                if let &Some(address) = opt_address {	                    	
                    mapping.add_mapping(new_ass, address.clone());
                } else {
                    unresolved.insert(new_ass.clone());
                    
                    if opt_mappings.len() > 1 {
                        // having a single None is fine
                        // probably a class method then
                        warning.insert(new_ass);                        
                    }
                }
            }
            
            // look for the attribute in its types
            if unresolved.len() > 0 {
                let types = parent_object.get_extension();
                
                if types.len() == 0 {
                    for unmet in unresolved.iter() {
                        //todo, add type information as well
                        error.insert(unmet.clone());
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
                                error.insert(new_ass);
                                continue;
                            } else {
                                mapping.add_mapping(new_ass, opt_address.unwrap());
                            }
                        }
                    }
                }
            } 
        }

        if warning.len() > 0 {
            let mut items = HashMap::new();

            items.insert("parent".to_owned(), MessageItem::String(parent.to_string()));
            items.insert("name".to_owned(), MessageItem::String(name.clone()));

            let mut ass_count = 0;
            for assumption in warning {
                items.insert(format!("assumption {}", ass_count), MessageItem::Assumption(assumption.clone()));
                ass_count += 1;
            }
            
            let message = Message::Warning {
                source: self.nodes.last().unwrap().clone(),
                kind: WATTRIBUTE_UNSAFE,
                content: items,
            };
            &CHANNEL.publish(message);
        }

        if error.len() > 0 {
            let mut items = HashMap::new();

            items.insert("parent".to_owned(), MessageItem::String(parent.to_string()));
            items.insert("name".to_owned(), MessageItem::String(name.clone()));

            let mut ass_count = 0;
            for assumption in error {
                items.insert(format!("assumption {}", ass_count), MessageItem::Assumption(assumption.clone()));
                ass_count += 1;
            }
            
            let message = Message::Error {
                source: self.nodes.last().unwrap().clone(),
                kind: EATTRIBUTE_INVALID,
                content: items,
            };
            &CHANNEL.publish(message);
        }
        
        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: total_changes,
            result: mapping,
        };
    }


    fn assign(&mut self, targets: &Vec<GastNode>, value: &GastNode) -> ExecutionResult {
        let value_execution = self.execute(value);

        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();

        let mut value_changes = value_execution.changes;
        let mut value_dependencies = value_execution.dependencies;
        let value_mapping = value_execution.result;

        total_changes.append(&mut value_changes);
        total_dependencies.append(&mut value_dependencies);

        for target in targets {
            let target_result = self.assign_to_target(target, &value_mapping);
            let mut target_dependencies = target_result.dependencies;
            let mut target_changes = target_result.changes;

            total_changes.append(&mut target_changes);
            total_dependencies.append(&mut target_dependencies);
        }

        // todo change to python None
        let mapping = Mapping::simple(Assumption::empty(), -1);

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: total_changes,
            result: mapping,
        };
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

        let result = parent_result.result;
        let dependencies = parent_result.dependencies;
        
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

            let current_assumption = self.assumptions.last().unwrap().clone();
            
            let mut parent_object = self.memory.get_object_mut(parent_address);
            parent_object.assign_attribute(attribute.clone(),
                                            current_assumption,
                                            mapping.clone())
        }

        //todo, resolving parent may have had changes/dependencies
        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: changes,
            result: Mapping::new(),
        };
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

        return ExecutionResult {
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

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![AnalysisItem::Identifier { name: target.clone() }],
            result: mapping,
        };
    }

    pub fn declare_new_constant(&mut self, target: &String, tpe: &String) -> ExecutionResult {
        let pointer = self.object_of_type(tpe);
        let mut scope = self.scopes.last_mut().unwrap();
        let mapping = Mapping::simple(Assumption::empty(), pointer);
        scope.set_constant(target.clone(),
                            self.assumptions.last().unwrap().clone(),
                            mapping.clone());

        let result = Mapping::simple(Assumption::empty(), NONE);
        return ExecutionResult {
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

    pub fn declare_sub_type(&mut self, name: &String, parent: &String) {
        let result = self.load_identifier(parent).result;
        let (_, parent_pointer) = result.iter().next().unwrap();

        let new_pointer = self.memory.new_object();
        {
            let mut object = self.memory.get_object_mut(&new_pointer);
            object.set_type(true);
            object.extend(parent_pointer.clone());
        }

        self.knowledge_base.add_type(name.clone(), new_pointer.clone());
        self.assign_to_identifier(name, &Mapping::simple(Assumption::empty(), new_pointer));
    }

    pub fn new_scope(&mut self) {
        self.scopes.push(Scope::new());
    }
}
