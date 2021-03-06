use core::*;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::btree_map::Entry;

pub struct PythonBinOp { }

impl BinOpExecutor for PythonBinOp {
    fn execute(&self,
               env: Environment,
               left: &GastNode,
               op: &str,
               right: &GastNode)
               -> ExecutionResult {
        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();
        let mut result = Mapping::new();

        let Environment { vm, executors } = env;

        let mut left_result = vm.execute(executors, left);
        let left_mapping = left_result.result;
        total_changes.append(&mut left_result.changes);
        total_dependencies.append(&mut left_result.dependencies);

        let mut right_result = vm.execute(executors, right);
        let right_mapping = right_result.result;
        total_changes.append(&mut right_result.changes);
        total_dependencies.append(&mut right_result.dependencies);

        let mut error = BTreeMap::new();

        for &(ref left_path, ref left_address) in &left_mapping {
            for &(ref right_path, ref right_address) in &right_mapping {
                if !left_path.mergeable(right_path) {
                    continue;
                }

                let mut new_path = left_path.clone();

                // todo can probably avoid this clone
                new_path.merge_into(right_path.clone());

                // todo, bit of a hack
                // concludes that if the most recently defined type supports addition
                // that the entire thing does
                // reality is more complicated, and full of runtime type checks
                let mut ancestor_name = "None".to_owned();

                for ancestor in vm.common_ancestor(left_address, right_address).iter().rev() {
                    ancestor_name = vm.knowledge().get_type_name(ancestor).clone();
                    if vm.knowledge().operation_supported(&ancestor_name, op) {
                        break;
                    }
                }

                // multiplying sequences with an integer is an annoying special case
                let tuple_type = *vm.knowledge().get_type(&"tuple".to_owned()).unwrap();  
                let list_type = *vm.knowledge().get_type(&"list".to_owned()).unwrap(); 
                let string_type = *vm.knowledge().get_type(&"str".to_owned()).unwrap(); 
                let int_type = *vm.knowledge().get_type(&"int".to_owned()).unwrap();

                let left_ancestors = vm.ancestors(left_address);
                let right_ancestors = vm.ancestors(right_address);

                let b1 = left_ancestors.contains(&tuple_type) 
                    || left_ancestors.contains(&list_type)
                    || left_ancestors.contains(&string_type);
                let b2 = right_ancestors.contains(&tuple_type) 
                    || right_ancestors.contains(&list_type)
                    || right_ancestors.contains(&string_type); 
                let b3 = right_ancestors.contains(&int_type);
                let b4 = left_ancestors.contains(&int_type);         

                // normal multiplication 
                if vm.knowledge().operation_supported(&ancestor_name, op) {
                    let new_type = match &ancestor_name[..] {
                        "number" => "float".to_owned(),
                        _ => ancestor_name,
                    };

                    let mut new_ptr = vm.object_of_type(&new_type);

                    

                    // + for concatenation
                    // sets use - for difference, just model it as a concatenation for now
                    if (op == "+" || op == "-") 
                        && vm.is_subtype(&new_type, "collection") {
                        // todo, dirty workaround for augmented assign
                        new_ptr = *left_address;

                        let new_col;
                        {
                            let left_obj = vm.get_object(left_address);
                            let right_obj = vm.get_object(right_address);
                            let left_col = left_obj.get_elements();
                            let right_col = right_obj.get_elements();
                            new_col = left_col.concatenate(right_col);
                        }

                        let mut new_object = vm.get_object_mut(&new_ptr);
                        new_object.set_elements(new_col);
                    }   

                    result.add_mapping(new_path, new_ptr);              
                } else if op == "*" && b1 && b3 {
                    let old_type;
                    let collection;
                    {
                        let old_object = vm.get_object(left_address);
                        old_type = *old_object.get_extension().first().unwrap();
                        collection = old_object.get_elements().clone();
                    }

                    let new_ptr = vm.object_of_type_pointer(&old_type);
                    result.add_mapping(new_path, new_ptr);
                    let mut new_object = vm.get_object_mut(&new_ptr);
                    new_object.set_elements(collection); 
                } else if op == "*" && b2 && b4 {
                    let old_type;
                    let collection;
                    {
                        let old_object = vm.get_object(right_address);
                        old_type = *old_object.get_extension().first().unwrap();
                        collection = old_object.get_elements().clone();
                    }

                    let new_ptr = vm.object_of_type_pointer(&old_type);
                    result.add_mapping(new_path, new_ptr);
                    let mut new_object = vm.get_object_mut(&new_ptr);
                    new_object.set_elements(collection); 
                } else {
                    let kb = vm.knowledge();
                    let left_object = vm.get_object(left_address);
                    let left_type = left_object.get_type_name(kb);
                    let right_object = vm.get_object(right_address);
                    let right_type = right_object.get_type_name(kb);

                    match error.entry((left_type, right_type)) {
                        Entry::Vacant(o) => {
                            let mut left_set = BTreeSet::new();
                            let mut right_set = BTreeSet::new();
                            left_set.insert(left_path.clone());
                            right_set.insert(right_path.clone());
                            o.insert((left_set, right_set));
                        }
                        Entry::Occupied(mut entry) => {
                            let &mut (ref mut left_set, ref mut right_set) = entry.get_mut();
                            left_set.insert(left_path.clone());
                            right_set.insert(right_path.clone());
                        }
                    }
                }
            }
        }

        if !error.is_empty() {
            let content = BinOpInvalid::new(op.to_owned(), error);
            let message = Message::Output { 
                source: vm.current_node().clone(),
                content: Box::new(content)};
            CHANNEL.publish(message);
        }

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: total_changes,
            result: result,
        }
    }
}
