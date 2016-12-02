use super::Pointer;
use super::Assumption;

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
        results: Vec<Result>,
    },
}

#[derive(Debug, Clone)]
pub struct Result {
    pub assumption: Assumption,
    pub value: Pointer,
}

#[derive(Debug, Clone)]
pub enum Change {
    Identifier { name: String },
    Object { address: Pointer },
}
