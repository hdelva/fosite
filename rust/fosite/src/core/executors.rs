use super::GastNode;
use super::ExecutionResult;
use super::VirtualMachine;
use super::KnowledgeBase;

pub struct Executors {
    pub binop: Option<Box<BinOpExecutor>>,
    pub boolop: Option<Box<BoolOpExecutor>>,
    pub conditional: Option<Box<ConditionalExecutor>>,
    pub block: Option<Box<BlockExecutor>>,
    pub identifier: Option<Box<IdentifierExecutor>>,
    pub attribute: Option<Box<AttributeExecutor>>,
    pub boolean: Option<Box<BooleanExecutor>>,
    pub string: Option<Box<StringExecutor>>,
    pub int: Option<Box<IntExecutor>>,
    pub float: Option<Box<FloatExecutor>>,
    pub declaration: Option<Box<DeclarationExecutor>>,
    pub assign: Option<Box<AssignExecutor>>,
    pub while_loop: Option<Box<WhileExecutor>>,
}

pub trait AssignExecutor {
    fn execute(&self,
               env: Environment,
               targets: &Vec<GastNode>,
               value: &GastNode)
               -> ExecutionResult;
}

pub trait DeclarationExecutor {
    fn execute(&self, env: Environment, name: &String, kind: &String) -> ExecutionResult;
}

pub trait IntExecutor {
    fn execute(&self, env: Environment) -> ExecutionResult;
}

pub trait FloatExecutor {
    fn execute(&self, env: Environment) -> ExecutionResult;
}

pub trait StringExecutor {
    fn execute(&self, env: Environment) -> ExecutionResult;
}

pub trait BooleanExecutor {
    fn execute(&self, env: Environment, value: bool) -> ExecutionResult;
}

pub trait AttributeExecutor {
    fn execute(&self, env: Environment, parent: &GastNode, name: &String) -> ExecutionResult;
}

pub trait IdentifierExecutor {
    fn execute(&self, env: Environment, name: &String) -> ExecutionResult;
}

pub trait BlockExecutor {
    fn execute(&self, env: Environment, content: &Vec<GastNode>) -> ExecutionResult;
}

pub trait BinOpExecutor {
    fn execute(&self,
               env: Environment,
               left: &GastNode,
               op: &String,
               right: &GastNode)
               -> ExecutionResult;
}

pub trait BoolOpExecutor {
    fn execute(&self,
               env: Environment,
               left: &GastNode,
               op: &String,
               right: &GastNode)
               -> ExecutionResult;
}

pub trait ConditionalExecutor {
    fn execute(&self,
               env: Environment,
               test: &GastNode,
               body: &GastNode,
               or_else: &GastNode)
               -> ExecutionResult;
}

pub trait WhileExecutor {
    fn execute(&self,
               env: Environment,
               test: &GastNode,
               body: &GastNode)
               -> ExecutionResult;
}

pub struct Environment<'a> {
    pub vm: &'a mut VirtualMachine,
    pub executors: &'a Executors,
}

impl<'a> Environment<'a> {
    pub fn new(vm: &'a mut VirtualMachine, executors: &'a Executors) -> Self {
        Environment {
            vm: vm,
            executors: executors,
        }
    }

    pub fn executors(&self) -> &Executors {
        self.executors
    }

    pub fn kb(&self) -> &KnowledgeBase {
        self.vm.knowledge()
    }

    pub fn vm(&self) -> &VirtualMachine {
        self.vm
    }

    pub fn kb_mut(&mut self) -> &mut KnowledgeBase {
        self.vm.knowledge_mut()
    }

    pub fn vm_mut(&mut self) -> &mut VirtualMachine {
        self.vm
    }
}
