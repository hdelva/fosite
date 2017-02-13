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

        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();

        let test_result = vm.execute(executors, test);

        let mut no = Vec::new();
        let mut yes = Vec::new();

        for change in test_result.changes.into_iter() {
            total_changes.push(change);
        }

        for dependency in test_result.dependencies.into_iter() {
            total_dependencies.push(dependency);
        }

        let t = vm.knowledge().constant(&"True".to_owned());
        let f = vm.knowledge().constant(&"False".to_owned());

        let total = test_result.result.len();

        // split up the test result into yes/no/maybe
        for (path, address) in test_result.result.into_iter() {
            if address == t {
                yes.push(path);
            } else if address == f {
                no.push(path);
            }
        }

        if no.len() == total {
            return self.strict_negative(vm, executors, or_else, total_changes, total_dependencies)
        } else if yes.len() == total {
            return self.strict_positive(vm, executors, body, total_changes, total_dependencies)
        } else {
            return self.branch(vm, executors, body, or_else, yes, no, total_changes, total_dependencies)
        }
    }
}

impl PythonConditional {
    fn branch(&self,
              vm: &mut VirtualMachine,
              executors: &Executors,
              body: &GastNode,
              or_else: &GastNode,
              yes: Vec<Path>,
              no: Vec<Path>,
              c: Vec<AnalysisItem>,
              d: Vec<AnalysisItem>) -> ExecutionResult {
                          
        let mut total_changes = HashSet::from_iter(c.into_iter());
        // rust can't infer the type?
        let mut total_dependencies: HashSet<_> = HashSet::from_iter(d.into_iter());

        let last_path = vm.pop_path();

        let mut positive = last_path.clone();
        positive.add_node(PathNode::Condition(vm.current_node(), true));
        let mut negative = last_path.clone();
        negative.add_node(PathNode::Condition(vm.current_node(), false));

        vm.push_path(positive);
        vm.add_restrictions(no);
        let body_result = vm.execute(executors, body);
        vm.drop_restrictions();
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
        vm.add_restrictions(yes);
        let else_result = vm.execute(executors, or_else);
        vm.drop_restrictions();
        let _ = vm.pop_path();

        let changes = else_result.changes;
        let dependencies = else_result.dependencies;

        for change in &changes {
            total_changes.insert(change.clone());
        }

        for dependency in &dependencies {
            total_dependencies.insert(dependency.clone());
        }

        vm.push_path(last_path);

        vm.merge_branches(&total_changes);

        self.check(vm, executors, &total_changes);

        // todo any way around the cloning?
        return ExecutionResult {
            changes: Vec::from_iter(total_changes.into_iter()),
            dependencies: Vec::from_iter(total_dependencies.into_iter()),
            flow: FlowControl::Continue,
            result: Mapping::new(),
        };
    }

    fn strict_positive(&self,
                       vm: &mut VirtualMachine,
                       executors: &Executors,
                       body: &GastNode,
                       changes: Vec<AnalysisItem>,
                       dependencies: Vec<AnalysisItem>) -> ExecutionResult {
        let last_path = vm.pop_path();

        let mut positive = last_path.clone();
        positive.add_node(PathNode::Condition(vm.current_node(), true));
        vm.push_path(positive);

        let result = self.strict(vm, executors, body, changes, dependencies);

        let _ = vm.pop_path();
        vm.push_path(last_path);

        vm.lift_branches(&result.changes);

        return result;
    }

    fn strict_negative(&self,
                       vm: &mut VirtualMachine,
                       executors: &Executors,
                       body: &GastNode,
                       changes: Vec<AnalysisItem>,
                       dependencies: Vec<AnalysisItem>) -> ExecutionResult {
        let last_path = vm.pop_path();

        let mut negative = last_path.clone();
        negative.add_node(PathNode::Condition(vm.current_node(), false));
        vm.push_path(negative);

        let result = self.strict(vm, executors, body, changes, dependencies);

        let _ = vm.pop_path();
        vm.push_path(last_path);

        return result;
    }

    fn strict(&self, 
              vm: &mut VirtualMachine, 
              executors: &Executors, 
              body: &GastNode,
              mut changes: Vec<AnalysisItem>,
              mut dependencies: Vec<AnalysisItem>) -> ExecutionResult {
        
        let result = vm.execute(executors, body);

        for change in result.changes.into_iter() {
            changes.push(change);
        }

        for dependency in result.dependencies.into_iter() {
            dependencies.push(dependency);
        }

        return ExecutionResult {
            changes: changes,
            dependencies: dependencies,
            flow: FlowControl::Continue,
            result: Mapping::new(),
        };
    }

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
                                     MessageItem::String(type_name.to_owned()));

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
