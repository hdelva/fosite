use super::Pointer;
use super::Assumption;

pub enum FlowControl {
    Continue,
    TerminateLoop,
    TerminateCall,
}

pub enum ExecutionResult {
    Failure,
    Success {
        flow: FlowControl,
        dependencies: Vec<String>,
        invalidations: Vec<String>,
        results: Vec<Result>,
    }
}

pub struct Result {
    pub assumption: Assumption,
    pub value: Pointer,
}
