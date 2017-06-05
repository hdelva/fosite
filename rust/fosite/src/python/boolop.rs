use core::*;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::btree_map::Entry;

pub struct PythonBoolOp { }

impl BoolOpExecutor for PythonBoolOp {
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

        let t = vm.knowledge().constant(&"True".to_owned());
        let f = vm.knowledge().constant(&"False".to_owned());
        
        let op: &str = &*op; 

        for &(ref left_path, ref left_address) in &left_mapping {
            for &(ref right_path, ref right_address) in &right_mapping {
                if !left_path.mergeable(right_path) {
                    continue;
                }

                let mut new_path = left_path.clone();

                // todo can probably avoid this clone
                new_path.merge_into(right_path.clone());

                let mut type_name = "None".to_owned();

                // the in operator isn't reflexive
                // bit of an annoying special case
                if op == "in" || op == "not in" {
                    let ancestors = vm.ancestors(right_address);

                    for ancestor in ancestors.iter().rev() {
                        type_name = vm.knowledge().get_type_name(ancestor).clone();
                        if vm.knowledge().operation_supported(&type_name, &op.to_owned()) {
                            break;
                        }
                    }
                } else {
                    // todo, bit of a hack
                    // concludes that if the most recently defined type supports addition
                    // that the entire thing does
                    // reality is more complicated, and full of runtime type checks
                    let ancestors = vm.common_ancestor(left_address, right_address);

                    for ancestor in ancestors.iter().rev() {
                        type_name = vm.knowledge().get_type_name(ancestor).clone();
                        if vm.knowledge().operation_supported(&type_name, &op.to_owned()) {
                            break;
                        }
                    }
                }

                if vm.knowledge().operation_supported(&type_name, &op.to_owned()) {
                    let new_object = match op {
                        "==" | "is" | ">=" | "<=" => {
                            if *left_address == *right_address {
                                t
                            } else {
                                vm.object_of_type(&"bool".to_owned())
                            }
                        },
                        "!=" | "is not" => {
                            if *left_address == *right_address {
                                f
                            } else {
                                vm.object_of_type(&"bool".to_owned())
                            }
                        },
                        _ => vm.object_of_type(&"bool".to_owned()),
                    };

                    result.add_mapping(new_path, new_object);
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
                content: Box::new(content),
            };
            CHANNEL.publish(message);
        }

        if result.is_empty() {
            let new_object = vm.object_of_type(&"bool".to_owned());
            result.add_mapping(Path::empty(), new_object);
        }

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: total_changes,
            result: result,
        }
    }
}
