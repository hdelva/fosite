use super::Pointer;
use super::Mapping;
use super::{GastNode, NodeType};

#[derive(Debug, Clone)]
pub enum FlowControl {
    Continue,
    TerminateLoop,
    TerminateCall, 
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub flow: FlowControl,
    pub dependencies: Vec<AnalysisItem>,
    pub changes: Vec<AnalysisItem>,
    pub result: Mapping,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnalysisItem {
    Identifier(String),
    Object(Pointer),
    Attribute(Box<AnalysisItem>, String),
}

impl AnalysisItem {
    pub fn to_string(&self) -> String {
        match self {
            &AnalysisItem::Identifier ( ref name ) => name.clone(),
            &AnalysisItem::Object ( ref address ) => format!("{}", address),
            &AnalysisItem::Attribute ( ref parent, ref name ) => {
                format!("{}.{}", parent.to_string(), name)
            }
        }
    }

    pub fn is_object(&self) -> bool {
        match self {
            &AnalysisItem::Object ( _ ) => true,
            _ => false,
        }
    }

    pub fn is_identifier(&self) -> bool {
        match self {
            &AnalysisItem::Identifier ( _ ) => true,
            _ => false,
        }
    }

    pub fn is_attribute(&self) -> bool {
        match self {
            &AnalysisItem::Attribute ( _, _ ) => true,
            _ => false,
        }
    }

    pub fn as_node(&self) -> GastNode {
        match self {
            &AnalysisItem::Identifier ( ref name ) => {
                return GastNode {
                    id: 0,
                    kind: NodeType::Identifier { name: name.clone() },
                }
            }
            &AnalysisItem::Attribute ( ref parent, ref name ) => {
                return GastNode {
                    id: 0,
                    kind: NodeType::Attribute {
                        parent: Box::new(parent.as_node()),
                        attribute: name.clone(),
                    },
                }
            }
            _ => panic!("This AnalysisItem does not correspond to a GastNode"),
        }
    }
}
