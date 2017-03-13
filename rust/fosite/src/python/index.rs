use core::*;

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

pub struct PythonIndex {

}

impl IndexExecutor for PythonIndex {
    fn execute(&self,
               env: Environment,
               target: &GastNode,
               index: &GastNode)
               -> ExecutionResult{
        let Environment { vm, executors } = env;

        // needed to add this indexation to the path
        let current_node = vm.current_node().clone();

        // mapping to return
        let mut result_mapping = Mapping::new();

        let target_result = vm.execute(executors, target);

        // update dependencies and changes
        let mut total_dependencies = Vec::new();
        let mut total_changes = Vec::new();

        let mut dependencies = target_result.dependencies;
        let mut changes = target_result.changes;
        total_dependencies.append(&mut dependencies);
        total_changes.append(&mut changes);

        let index_result = vm.execute(executors, index);
        let mut dependencies = index_result.dependencies;
        let mut changes = index_result.changes;
        total_dependencies.append(&mut dependencies);
        total_changes.append(&mut changes);

        // target object b here
        let target_mapping = target_result.result;

        // index out of bounds warnings
        let mut warnings = Vec::new();

        // indexing something that doesn't support indexing
        let mut errors = BTreeMap::new();

        for (target_path, target_address) in target_mapping.iter() {
            // does this type of object support indexation?
            {
                let kb = vm.knowledge();
                let collection_type = kb.get_type(&"collection".to_owned()).unwrap();
                let set_type = kb.get_type(&"set".to_owned()).unwrap();
                let types = vm.ancestors(&target_address);
                
                if !(types.contains(collection_type)) || types.contains(set_type) {
                    let target_object = vm.get_object(target_address);
                    let type_name = target_object.get_type_name(kb);

                    match errors.entry(type_name.clone()) {
                        Entry::Vacant(v) => {
                            v.insert(vec![target_path.clone()]);
                        }
                        Entry::Occupied(mut o) => {
                            o.get_mut().push(target_path.clone());
                        }
                    };

                    continue;
                }
            }

            // we obviously depend on the target object
            total_dependencies.push(AnalysisItem::Object(target_address.clone()));

            let mut target_object = vm.get_object(target_address);
            let mut is_dict = false;
            {
                let kb = vm.knowledge();
                let dict_type = kb.get_type(&"dict".to_owned()).unwrap();
                if target_object.get_extension().contains(dict_type) {
                    let pls = target_object.get_attribute(&"___values".to_owned());
                    let (_, new_address) = pls.iter().next().unwrap();
                    target_object = vm.get_object(&new_address.unwrap());
                    is_dict = true;
                }
            }
            
            let value_mappings;
            // getting a fixed value can be done more accurately on sequences
            match (is_dict, &index.kind) {
                (false, &NodeType::Int {ref value}) => {
                    let adjusted_value;
                    if *value >= 0 {
                        // +1 because `Collection::first_combinations` starts at 1, not 0
                        adjusted_value = *value + 1;
                    } else {
                        adjusted_value = *value;
                    }

                    for (path, _, max) in target_object.size_range().into_iter() {
                        if let Some(max) = max {
                            if adjusted_value.abs() as usize > max {
                                warnings.push((path, max as i16));
                            }
                        }
                    }

                    value_mappings = target_object.get_element(adjusted_value as i16, &current_node);
                },
                _ => {
                    value_mappings = target_object.get_any_element(&current_node);
                }
            }
            
            // add all the possible values to the result map
            for (path, address) in value_mappings.iter() {
                // is it always mergeable?
                if target_path.mergeable(&path) {
                    let mut new_path = target_path.clone();

                    // todo can probably remove this clone
                    new_path.merge_into(path.clone());
                    result_mapping.add_mapping(new_path, *address);
                }
            }
        }

        if warnings.len() > 0 {
            let content = OutOfBounds::new(target.to_string(), warnings);
            let message = Message::Output {
                source: vm.current_node(),
                content: Box::new(content),
            };
            &CHANNEL.publish(message);
        }

        if errors.len() > 0 {
            let content = IndexInvalid::new(target.to_string(), errors);
            let message = Message::Output {
                source: vm.current_node(),
                content: Box::new(content),
            };
            &CHANNEL.publish(message);
        }
        
        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: total_changes,
            result: result_mapping,
        };

        return execution_result;
            
    }
}