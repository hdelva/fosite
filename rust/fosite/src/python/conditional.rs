use core::*;

use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::iter::FromIterator;

pub struct PythonConditional { }

impl ConditionalExecutor for PythonConditional {
    fn execute(&self,
               env: Environment,
               test: &GastNode,
               body: &GastNode,
               or_else: &GastNode)
               -> ExecutionResult {
        let Environment { vm, executors } = env;

        // todo execute the test properly
        let _ = vm.execute(executors, test);

        let last_path = vm.pop_path();

        let mut total_changes = HashSet::new();
        let mut total_dependencies = HashSet::new();

        let mut positive = last_path.clone();
        positive.add_node(PathNode::Condition(vm.current_node(), true));
        let mut negative = last_path.clone();
        negative.add_node(PathNode::Condition(vm.current_node(), false));

        vm.push_path(positive);
        let body_result = vm.execute(executors, body);
        let _ = vm.pop_path();

        let mut identifier_changed = false;

        let changes = body_result.changes;
        let dependencies = body_result.dependencies;

        for change in &changes {
            total_changes.insert(change.clone());

            if let &AnalysisItem::Identifier { .. } = change {
                identifier_changed = true;
            }
        }

        for dependency in &dependencies {
            total_dependencies.insert(dependency.clone());
        }

        vm.change_branch(identifier_changed, &total_changes);

        vm.push_path(negative);
        let else_result = vm.execute(executors, or_else);
        let _ = vm.pop_path();

        let changes = else_result.changes;
        let dependencies = else_result.dependencies;

        for change in &changes {
            total_changes.insert(change.clone());

            if let &AnalysisItem::Identifier { .. } = change {
                identifier_changed = true;
            }
        }

        for dependency in &dependencies {
            total_dependencies.insert(dependency.clone());
        }

        vm.push_path(last_path);

        vm.merge_branches(identifier_changed, &total_changes);

        self.check(vm, executors, &total_changes);

        // todo any way around the cloning?
        return ExecutionResult {
            changes: Vec::from_iter(total_changes.into_iter()),
            dependencies: Vec::from_iter(total_dependencies.into_iter()),
            flow: FlowControl::Continue,
            result: Mapping::new(),
        };
    }
}

impl PythonConditional {
    fn check(&self,
             vm: &mut VirtualMachine,
             executors: &Executors,
             changes: &HashSet<AnalysisItem>) {
        for change in changes {
            if !change.is_object() {
                let mut all_types = HashMap::new();

                let execution_result = match change {
                    &AnalysisItem::Identifier { ref name } => vm.load_identifier(executors, name),
                    &AnalysisItem::Attribute { ref parent, ref name } => {
                        vm.load_attribute(executors, &parent.as_node(), name)
                    }
                    _ => {
                        unreachable!("AnalysisItem is an object when a previous check should've \
                                      excluded this")
                    }
                };

                let result = execution_result.result;
                for (path, address) in result.iter() {
                    let object = vm.get_object(address);
                    let tpe = object.get_extension()[0];

                    match all_types.entry(tpe.clone()) {
                        Entry::Vacant(v) => {
                            v.insert(vec![path.clone()]);
                        }
                        Entry::Occupied(mut o) => {
                            o.get_mut().push(path.clone());
                        }
                    };
                }

                if all_types.len() > 1 {
                    let mut items = HashMap::new();

                    items.insert("name".to_owned(), MessageItem::String(change.to_string()));

                    let mut type_count = 0;
                    for (tpe, paths) in all_types {
                        let type_name = vm.knowledge().get_type_name(&tpe);
                        items.insert(format!("type {}", type_count),
                                     MessageItem::String(type_name.clone()));

                        let mut path_count = 0;
                        for path in paths {
                            items.insert(format!("type {} path {}", type_count, path_count),
                                         MessageItem::Path(path.clone()));
                            path_count += 1;
                        }
                        type_count += 1;
                    }

                    let kind = if change.is_identifier() {
                        WIDENTIFIER_POLY_TYPE
                    } else {
                        WATTRIBUTE_POLY_TYPE
                    };

                    let message = Message::Warning {
                        source: vm.current_node(),
                        kind: kind,
                        content: items,
                    };

                    &CHANNEL.publish(message);
                }
            }
        }
    }
}
