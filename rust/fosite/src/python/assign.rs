use core::*;

use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

pub struct PythonAssign { }

impl AssignExecutor for PythonAssign {
    fn execute(&self,
               env: Environment,
               targets: &Vec<GastNode>,
               value: &GastNode)
               -> ExecutionResult {
        let Environment { vm, executors } = env;

        let value_execution = vm.execute(executors, value);

        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();

        let mut value_changes = value_execution.changes;
        let mut value_dependencies = value_execution.dependencies;
        let value_mapping = value_execution.result;

        total_changes.append(&mut value_changes);
        total_dependencies.append(&mut value_dependencies);

        for target in targets {
            let target_result = self.assign_to_target(vm, executors, target, &value_mapping);
            let mut target_dependencies = target_result.dependencies;
            let mut target_changes = target_result.changes;

            total_changes.append(&mut target_changes);
            total_dependencies.append(&mut target_dependencies);
        }

        let mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: total_changes,
            result: mapping,
        };
    }
}

impl PythonAssign {
    fn assign_to_target(&self,
                        vm: &mut VirtualMachine,
                        executors: &Executors,
                        target: &GastNode,
                        mapping: &Mapping)
                        -> ExecutionResult {
        match &target.kind {
            &NodeType::Identifier { ref name } => {
                self.assign_to_identifier(vm, executors, name, mapping)
            }
            &NodeType::List { ref content } |
            &NodeType::Sequence { ref content } => {
                self.assign_to_iterable(vm, executors, content, mapping)
            },
            &NodeType::Attribute { ref parent, ref attribute } => {
                self.assign_to_attribute(vm, executors, parent, attribute, mapping)
            }
            &NodeType::Index {ref target, ref index} => {
                self.assign_to_index(vm, executors, target, index, mapping)
            }
            _ => panic!("unimplemented"),
        }
    }

    fn _assign_to_iterable<F>(&self, 
                           vm: &mut VirtualMachine,
                           executors: &Executors,
                           content: &[GastNode], 
                           mapping: &Mapping,
                           num: usize,
                           fun: F)
                           -> ExecutionResult
                           where F: Fn(&Object) -> Vec<Mapping> {
        let mut dependencies = Vec::new();
        let mut changes = Vec::new();

        let mut value_mappings = Vec::new();

        for (path, address) in mapping.iter() {
            let object = vm.get_object(address);

            for (_, min, max) in object.size_range() {
                if let Some(max) = max {
                    if max < num {
                        //todo warning
                        //blacklist this path
                    }
                } 

                if let Some(min) = min {
                    if min > num {
                        //todo warning
                        //blacklist this path
                    }
                }
            }

            // needs current node information to form the path
            let possibilities = fun(object);

            // iterate over all possible elements
            // combine the elements's path with the object's path 
            // store the results in value_mappings 
            //
            // maintain the same order
            // the first element of value_mappings contains 
            // the the mapping for the first assign target
            for (index, mapping) in possibilities.into_iter().enumerate() {
                if value_mappings.len() <= index {
                    value_mappings.push(Mapping::new());
                }

                let ref mut new_mapping = value_mappings[index];

                for (element_path, address) in mapping.into_iter() {
                    let mut new_path = vm.current_path().clone();
                    new_path.merge_into(element_path.clone());
                    new_path.merge_into(path.clone());
                    new_mapping.add_mapping(new_path, address);
                }
            }
        }

        for (target_mapping, target) in value_mappings.iter().zip(content) {
            let mut partial_result = self.assign_to_target(vm, executors, target, &target_mapping);
            changes.append(&mut partial_result.changes);
            dependencies.append(&mut partial_result.dependencies);
        }

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        };
    }

    fn slice(&self, 
              vm: &mut VirtualMachine,
              mapping: &Mapping,
              left: i16,
              right: i16)
              -> Mapping {
        let mut result_mapping = Mapping::new();

        for (path, address) in mapping.iter() {
            let elements;
            {
                let object = vm.get_object(address);
                // one way to fix an off-by-one error
                elements = object.slice_elements(left, right);
            }

            let type_name = "list".to_owned();
            let slice_ptr = vm.object_of_type(&type_name);
            {
                let mut object = vm.get_object_mut(&slice_ptr);
                let mut collection = Collection::new();
                collection.set_content(elements);
                object.set_elements(collection);
            }

            result_mapping.add_mapping(path.clone(), slice_ptr);
        }

        return result_mapping;
    }

    fn assign_to_iterable(&self, 
                          vm: &mut VirtualMachine,
                          executors: &Executors,
                          content: &[GastNode], 
                          mapping: &Mapping) 
                          -> ExecutionResult {
        let mut dependencies = Vec::new();
        let mut changes = Vec::new();

        let current_node = vm.current_node().clone();

        let num = content.len();

        let mut split = None;
        for (index, target) in content.iter().enumerate() {
            let &GastNode {ref kind, ..} = target;
            if let &NodeType::UnOp { ref value, .. } = kind {
                split = Some((index, value));
            }
        }

        if let Some((index, target)) = split {
            let fun = |obj: &Object| {
                obj.get_first_n_elements(index as i16, &current_node)
            };

            let mut partial_result = self._assign_to_iterable(vm, executors, &content[..index], mapping, index, fun);
            changes.append(&mut partial_result.changes);
            dependencies.append(&mut partial_result.dependencies);

            let pls = num - index - 1;

            let fun = |obj: &Object| {
                obj.get_last_n_elements(pls as i16, &current_node)
            };

            let mut partial_result = self._assign_to_iterable(vm, executors, &content[index + 1..], mapping, pls, fun);
            changes.append(&mut partial_result.changes);
            dependencies.append(&mut partial_result.dependencies);

            let slice_mapping = self.slice(vm, mapping, index as i16, pls as i16);
            let mut partial_result = self.assign_to_target(vm, executors, target, &slice_mapping);
            changes.append(&mut partial_result.changes);
            dependencies.append(&mut partial_result.dependencies);
        } else {
            let fun = |obj: &Object| {
                obj.get_first_n_elements(num as i16, &current_node)
            };

            let mut partial_result = self._assign_to_iterable(vm, executors, content, mapping, num, fun);
            changes.append(&mut partial_result.changes);
            dependencies.append(&mut partial_result.dependencies);
        }

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        };
    }

    fn insert_dictionary(&self, 
        vm: &mut VirtualMachine, 
        target_address: &Pointer, 
        index_mapping: &Mapping,
        value_mapping: &Mapping) {

        // todo this clone shouldn't be necessary
        let current_path = vm.current_path().clone();

        let keys;
        let values;
        {
            let target_object = vm.get_object(target_address);

            keys = target_object.get_attribute(&"___keys".to_owned()).clone();
            values = target_object.get_attribute(&"___values".to_owned()).clone();
        }
        

        let mut key_chunk = CollectionChunk::empty();
        let mut value_chunk = CollectionChunk::empty();

        for (path, pointer) in value_mapping.iter() {
            let value_obj = vm.get_object(pointer);
            let kind = value_obj.get_extension().first().unwrap();

            value_chunk.add_representant(path.clone(), Representant::new(pointer.clone(), kind.clone(), Some(0), Some(1)));
        }

        for (path, pointer) in index_mapping.iter() {
            let value_obj = vm.get_object(pointer);
            let kind = value_obj.get_extension().first().unwrap();

            key_chunk.add_representant(path.clone(), Representant::new(pointer.clone(), kind.clone(), Some(0), Some(1)));
        }

        for (_, opt_address) in values.iter() {
            if let &Some(ref address) = opt_address {
                // add the new element
                let mut parent_object = vm.get_object_mut(address);

                // add the new element
                parent_object.append_element(value_chunk.clone(), current_path.clone());
            }
        }
        
        for (_, opt_address) in keys.iter() {
            if let &Some(ref address) = opt_address {
                // add the new element
                let mut parent_object = vm.get_object_mut(address);

                // add the new element
                parent_object.append_element(key_chunk.clone(), current_path.clone());
            }
        }
    }

    fn insert_collection(&self, 
        vm: &mut VirtualMachine, 
        target: &GastNode,
        target_address: &Pointer, 
        mapping: &Mapping) {

        // todo this clone shouldn't be necessary
        let current_path = vm.current_path().clone();

        let mut chunk = CollectionChunk::empty();

        let mut max = Some(1);
        for node in current_path.iter().rev() {
            match node {
                &PathNode::Loop(_, _, _) => {
                    max = None;
                    break;
                },
                &PathNode::Frame(_, _, _, _) => {
                    break;
                },
                _ => ()
            }
        }

        for (path, pointer) in mapping.iter() {
            let value_obj = vm.get_object(pointer);
            let kind = value_obj.get_extension().first().unwrap();

            chunk.add_representant(path.clone(), Representant::new(pointer.clone(), kind.clone(), Some(0), max));
        }

        // remember the type of the collection before the addition
        let original_type;
        {
            let parent_object = vm.get_object(target_address);
            let kb = vm.knowledge();
            original_type = parent_object.get_type_name(kb);
        }

        // add the new element
        {
            let mut parent_object = vm.get_object_mut(target_address);

            // add the new element
            parent_object.insert_element(chunk, current_path.clone());
        }

        // get the new type of the
        let new_type;
        {
            let parent_object = vm.get_object(target_address);
            let kb = vm.knowledge();
            new_type = parent_object.get_type_name(kb);
        }

        // check whether or not an element of a new type had been added
        if !new_type.contains(&original_type) {
            let content = HeteroCollection::new(target.to_string(), original_type, new_type);
            let message = Message::Output {
                source: vm.current_node().clone(),
                content: Box::new(content),
            };
            &CHANNEL.publish(message);
        }
    }

    fn assign_to_index(&self,
                           vm: &mut VirtualMachine,
                           executors: &Executors,
                           target: &GastNode,
                           index: &GastNode,
                           mapping: &Mapping)
                           -> ExecutionResult {
        // todo get rid of clone
        let mapping = mapping.clone().augment(PathNode::Assignment(vm.current_node().clone(),
                                                                   format!("{}[{}]",
                                                                           target.to_string(),
                                                                           index.to_string())));        

        let target_result = vm.execute(executors, target);
        let target_mapping = target_result.result;
        let mut dependencies = target_result.dependencies;
        let mut changes = target_result.changes;

        let mut index_result = vm.execute(executors, index);
        let index_mapping = index_result.result;
        dependencies.append(&mut index_result.dependencies);
        changes.append(&mut index_result.changes);

        let mut errors = BTreeMap::new();

        // add the object changes
        // perform the assignment
        for (target_path, target_address) in target_mapping.iter() {
            // does this type of object support item assignment?
            {
                let seq_type;
                let dict_type;

                let mut new_path = vm.current_path().clone();
                new_path.merge_into(target_path.clone());

                {
                    let kb = vm.knowledge();
                    seq_type = kb.get_type(&"mutable_sequence".to_owned()).unwrap().clone();
                    dict_type = kb.get_type(&"dict".to_owned()).unwrap().clone();
                }

                let types = vm.ancestors(&target_address);
                
                if types.contains(&seq_type) { 
                    changes.push(AnalysisItem::Object(target_address.clone()));

                    vm.store_object_change(target_address.clone(), &new_path);

                    // todo, pass on the new_path
                    self.insert_collection(vm, target, target_address, &mapping);
                }
                else if types.contains(&dict_type) {
                    changes.push(AnalysisItem::Object(target_address.clone()));

                    vm.store_object_change(target_address.clone(), &new_path);

                    // todo, pass on the new_path
                    self.insert_dictionary(vm, target_address, &index_mapping, &mapping);
                }
                else {
                    let kb = vm.knowledge();
                    let target_object = vm.get_object(target_address);
                    let type_name = target_object.get_type_name(kb);

                    match errors.entry(type_name.clone()) {
                        Entry::Vacant(v) => {
                            v.insert(vec![new_path]);
                        }
                        Entry::Occupied(mut o) => {
                            o.get_mut().push(new_path);
                        }
                    };
                }
            }   
        }

        if errors.len() > 0 {
            let content = InsertInvalid::new(target.to_string(), errors);
            let message = Message::Output {
                source: vm.current_node().clone(),
                content: Box::new(content),
            };
            &CHANNEL.publish(message);
        }

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        };
    }

    fn assign_to_attribute(&self,
                           vm: &mut VirtualMachine,
                           executors: &Executors,
                           parent: &GastNode,
                           attribute: &String,
                           mapping: &Mapping)
                           -> ExecutionResult {

        // todo get rid of clone
        let mapping = mapping.clone().augment(PathNode::Assignment(vm.current_node().clone(),
                                                                   format!("{}.{}",
                                                                           parent.to_string(),
                                                                           attribute)));

        let parent_result = vm.execute(executors, parent);

        let result = parent_result.result;
        let dependencies = parent_result.dependencies;

        let parent_mapping = result;
        let mut changes = Vec::new();

        // add the object changes
        // perform the assignment
        for (parent_path, parent_address) in parent_mapping.iter() {
            // todo this clone shouldn't be necessary
            let mut new_path = vm.current_path().clone();
            new_path.merge_into(parent_path.clone());

            changes.push(AnalysisItem::Object(parent_address.clone()));

            vm.store_object_change(parent_address.clone(), &new_path);

            let mut parent_object = vm.get_object_mut(parent_address);
            parent_object.assign_attribute(attribute.clone(), new_path, mapping.clone())
        }

        if let Some(item) = parent.kind.to_analysis_item() {
            changes.push(AnalysisItem::Attribute (
                Box::new(item.clone()),
                attribute.clone(),
            ));

            let path = &vm.current_path().clone();
            
            vm.store_identifier_change(AnalysisItem::Attribute (
                Box::new(item),
                attribute.clone(),
            ), &path, &parent_mapping);
        }

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        };
    }

    fn assign_to_identifier(&self,
                            vm: &mut VirtualMachine,
                            _: &Executors,
                            target: &String,
                            mapping: &Mapping)
                            -> ExecutionResult {
        let changes = vec![AnalysisItem::Identifier (target.clone())];

        // todo get rid of clone
        let mapping = mapping.clone()
            .augment(PathNode::Assignment(vm.current_node().clone(), target.clone()));

        let path = vm.current_path().clone();

        vm.store_identifier_change(AnalysisItem::Identifier(target.clone()), &path, &mapping);

        {
            let mut scope = vm.last_scope_mut();
            scope.set_mapping(target.clone(), path, mapping.clone());
        }

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: changes,
            result: result_mapping,
        };
    }
}
