use core::*;

use std::collections::HashMap;
use std::collections::BTreeSet;

pub struct PythonIdentifier { }

impl IdentifierExecutor for PythonIdentifier {
    fn execute(&self, env: Environment, name: &String) -> ExecutionResult {
        let mut unresolved = BTreeSet::new();
        unresolved.insert(Path::empty());

        let mut mapping = Mapping::new();

        let mut warning = BTreeSet::new();

        for scope in env.vm().scopes().rev() {
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
            let mut items = HashMap::new();

            items.insert("name".to_owned(), MessageItem::String(name.clone()));

            let mut path_count = 0;
            for path in warning {
                items.insert(format!("path {}", path_count),
                             MessageItem::Path(path.clone()));
                path_count += 1;
            }

            let message = Message::Warning {
                source: env.vm().current_node(),
                kind: WIDENTIFIER_UNSAFE,
                content: items,
            };
            &CHANNEL.publish(message);
        }

        if unresolved.len() > 0 {
            let mut items = HashMap::new();

            items.insert("name".to_owned(), MessageItem::String(name.clone()));

            let mut path_count = 0;
            for path in unresolved {
                items.insert(format!("path {}", path_count),
                             MessageItem::Path(path.clone()));
                path_count += 1;
            }

            let message = Message::Error {
                source: env.vm().current_node(),
                kind: EIDENTIFIER_INVALID,
                content: items,
            };
            &CHANNEL.publish(message);
        }

        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![AnalysisItem::Identifier { name: name.clone() }],
            changes: vec![],
            result: mapping,
        };

        return execution_result;

    }
}
