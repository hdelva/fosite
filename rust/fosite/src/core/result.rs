use super::Pointer;
use super::Mapping;

#[derive(Debug, Clone)]
pub enum FlowControl {
    Continue,
    TerminateLoop,
    TerminateCall,
}

#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Failure,
    Success {
        flow: FlowControl,
        dependencies: Vec<AnalysisItem>,
        changes: Vec<AnalysisItem>,
        result: Mapping,
    },
}

#[derive(Debug, Clone)]
pub enum AnalysisItem {
    Identifier { name: String },
    Object { address: Pointer },
    Attribute { parent: Box<AnalysisItem>, name: String },
}
