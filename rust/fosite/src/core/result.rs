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
        dependencies: Vec<String>,
        changes: Vec<Change>,
        result: Mapping,
    },
}

#[derive(Debug, Clone)]
pub enum Change {
    Identifier { name: String },
    Object { address: Pointer },
}
