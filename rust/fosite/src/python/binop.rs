use core::*;

use std::collections::HashMap;
use std::collections::BTreeSet;
use std::collections::hash_map::Entry;

pub struct PythonBinOp { }

impl BinOpExecutor for PythonBinOp {
    fn execute(&self,
               env: Environment,
               left: &GastNode,
               op: &String,
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

        let mut error = HashMap::new();

        for (left_path, left_address) in left_mapping.iter() {
            for (right_path, right_address) in right_mapping.iter() {
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
                    ancestor_name = vm.knowledge().get_type_name(&ancestor).clone();
                    if vm.knowledge().operation_supported(&ancestor_name, op) {
                        break;
                    }
                }

                if vm.knowledge().operation_supported(&ancestor_name, op) {
                    let new_type = match &ancestor_name[..] {
                        "number" => "float".to_owned(),
                        _ => ancestor_name,
                    };

                    let new_object = vm.object_of_type(&new_type);

                    result.add_mapping(new_path, new_object);
                } else {
                    let left_object = vm.get_object(left_address);
                    let left_type = left_object.get_extension().first().unwrap();
                    let left_type_name = vm.knowledge().get_type_name(left_type).clone();
                    let right_object = vm.get_object(right_address);
                    let right_type = right_object.get_extension().first().unwrap();
                    let right_type_name = vm.knowledge().get_type_name(right_type).clone();

                    match error.entry((left_type_name, right_type_name)) {
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

        if error.len() > 0 {
            let mut items = HashMap::new();

            items.insert("operation".to_owned(), MessageItem::String(op.clone()));

            let mut comb_count = 0;
            for (types, paths) in error {
                let (left_type, right_type) = types;

                items.insert(format!("combination {} left", comb_count),
                             MessageItem::String(left_type.clone()));
                items.insert(format!("combination {} right", comb_count),
                             MessageItem::String(right_type.clone()));

                let (left_paths, right_paths) = paths;

                let mut path_count = 0;
                for path in left_paths.into_iter() {
                    items.insert(format!("combination {} left {}", comb_count, path_count),
                                 MessageItem::Path(path));
                    path_count += 1;
                }

                let mut path_count = 0;
                for path in right_paths.into_iter() {
                    items.insert(format!("combination {} right {}", comb_count, path_count),
                                 MessageItem::Path(path));
                    path_count += 1;
                }

                comb_count += 1;
            }

            let message = Message::Error {
                source: vm.current_node(),
                kind: EBINOP,
                content: items,
            };
            &CHANNEL.publish(message);
        }

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: total_changes,
            result: result,
        };

        return execution_result;
    }
}
