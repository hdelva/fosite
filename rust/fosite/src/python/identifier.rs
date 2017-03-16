use core::*;

use std::collections::BTreeSet;

pub struct PythonIdentifier { }

impl IdentifierExecutor for PythonIdentifier {
    fn execute(&self, env: Environment, name: &String) -> ExecutionResult {
        let Environment { vm, .. } = env;

        let mut unresolved = BTreeSet::new();
        unresolved.insert(Path::empty());

        let mut mapping = Mapping::new();

        let mut warning = BTreeSet::new();

        for scope in vm.scopes().rev() {
            let opt_mappings = scope.resolve_optional_identifier(&name);

            let mut new_unresolved = BTreeSet::new();

            for (path, opt_address) in opt_mappings.iter() {
                for unresolved_path in &unresolved {
                    let mut new_path = path.clone();
                    for pls in unresolved_path.iter() {
                        new_path.add_node(pls.clone());
                    }

                    if let &Some(address) = opt_address {
                        mapping.add_mapping(new_path, address.clone());
                    } else {
                        new_unresolved.insert(new_path.clone());

                        if opt_mappings.len() > 1 {
                            warning.insert(new_path);
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
            let content = IdentifierUnsafe::new(name.clone(), warning);
            let message = Message::Output {
                source: vm.current_node().clone(), 
                content: Box::new(content),
            };
            &CHANNEL.publish(message);
        }

        if unresolved.len() > 0 {
            let content = IdentifierInvalid::new(name.clone(), unresolved);
            let message = Message::Output {
                source: vm.current_node().clone(), 
                content: Box::new(content),
            };
            &CHANNEL.publish(message);
        }

        vm.store_identifier_dependency(AnalysisItem::Identifier(name.clone()), &mapping);

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![AnalysisItem::Identifier(name.clone())],
            changes: vec![],
            result: mapping,
        };

        return execution_result;
    }
}
