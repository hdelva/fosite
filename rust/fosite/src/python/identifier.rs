use core::*;

use std::collections::BTreeSet;

pub struct PythonIdentifier { }

impl IdentifierExecutor for PythonIdentifier {
    fn execute(&self, env: Environment, name: &String) -> ExecutionResult {
        let Environment { vm, .. } = env;

        let mut unresolved = BTreeSet::new();
        unresolved.insert(Path::empty());

        let mut mapping = Mapping::new();

        for scope in vm.scopes().rev() {
            let opt_mappings = scope.resolve_optional_identifier(&name);

            let mut new_unresolved = BTreeSet::new();

            for &(ref path, ref opt_address) in opt_mappings.iter() {
                for unresolved_path in &unresolved {
                    let mut new_path = path.clone();
                    for pls in unresolved_path.iter() {
                        new_path.add_node(pls.clone());
                    }

                    if let &Some(address) = opt_address {
                        mapping.add_mapping(new_path, address.clone());
                    } else {
                        new_unresolved.insert(new_path.clone());
                    }
                }
            }

            unresolved = new_unresolved;
            if unresolved.len() == 0 {
                break;
            }
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

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![AnalysisItem::Identifier(name.clone())],
            changes: vec![],
            result: mapping,
        }
    }
}
