use super::Pointer;
use super::Assumption;

#[derive(Debug)]
pub enum FlowControl {
    Continue,
    TerminateLoop,
    TerminateCall,
}

#[derive(Debug)]
pub enum ExecutionResult {
    Failure,
    Success {
        flow: FlowControl,
        dependencies: Vec<String>,
        changes: Vec<Change>,
        results: Vec<Result>,
    },
}

#[derive(Debug)]
pub struct Result {
    pub assumption: Assumption,
    pub value: Pointer,
}

#[derive(Debug)]
pub enum Change {
    Identifier { name: String },
    Object { address: Pointer },
}
