use super::GastNode;
use super::ExecutionResult;
use super::VirtualMachine;
use super::KnowledgeBase;
use super::Pointer;
use super::Mapping;

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
    pub break_loop: Option<Box<BreakExecutor>>,
    pub continue_loop: Option<Box<ContinueExecutor>>,
    pub list: Option<Box<ListExecutor>>,
    pub sequence: Option<Box<SequenceExecutor>>,
    pub index: Option<Box<IndexExecutor>>,
    pub set: Option<Box<SetExecutor>>,
    pub dict: Option<Box<DictExecutor>>,
    pub generator: Option<Box<GeneratorExecutor>>,
    pub filter: Option<Box<FilterExecutor>>,
    pub map: Option<Box<MapExecutor>>,
    pub andthen: Option<Box<AndThenExecutor>>,
    pub foreach: Option<Box<ForEachExecutor>>,
    pub call: Option<Box<CallExecutor>>,
    pub method: Option<Box<MethodExecutor>>,
    pub import: Option<Box<ImportExecutor>>,
    pub negate: Option<Box<NegateExecutor>>,
    pub unop: Option<Box<UnOpExecutor>>,
    pub slice: Option<Box<SliceExecutor>>,
    pub function: Option<Box<FunctionDefExecutor>>,
}

pub trait MethodExecutor {
    fn execute(&self,
               env: Environment,
               parent: &Pointer,
               address: &Pointer)
               -> Pointer;
}

pub trait SliceExecutor {
    fn execute(&self,
               env: Environment,
               target: &GastNode,
               lower: &GastNode,
               upper: &GastNode)
               -> ExecutionResult;
}

pub trait AssignExecutor {
    fn execute(&self,
               env: Environment,
               targets: &Vec<GastNode>,
               value: &GastNode)
               -> ExecutionResult;

    fn direct(&self,
               env: Environment,
               target: String,
               value: Mapping)
               -> ExecutionResult;
}

pub trait UnOpExecutor {
    fn execute(&self,
               env: Environment,
               value: &GastNode)
               -> ExecutionResult;
}

pub trait NegateExecutor {
    fn execute(&self,
               env: Environment,
               value: &GastNode)
               -> ExecutionResult;
}

pub trait ImportExecutor {
    fn execute(&self,
               env: Environment,
               module: &String,
               parts: &Vec<(String, String)>,
               into: &Option<String>)
               -> ExecutionResult;
}

pub trait IndexExecutor {
    fn execute(&self,
               env: Environment,
               target: &GastNode,
               index: &GastNode)
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

pub trait BreakExecutor {
    fn execute(&self, env: Environment) -> ExecutionResult;
}

pub trait ContinueExecutor {
    fn execute(&self, env: Environment) -> ExecutionResult;
}

pub trait StringExecutor {
    fn execute(&self, env: Environment) -> ExecutionResult;
}

pub trait BooleanExecutor {
    fn execute(&self, env: Environment, value: bool) -> ExecutionResult;
}

pub trait ListExecutor {
    fn execute(&self, env: Environment, content: &Vec<GastNode>) -> ExecutionResult;
}

pub trait SetExecutor {
    fn execute(&self, env: Environment, content: &Vec<GastNode>) -> ExecutionResult;
}

pub trait DictExecutor {
    fn execute(&self, env: Environment, content: &Vec<GastNode>) -> ExecutionResult;
}

pub trait SequenceExecutor {
    fn execute(&self, env: Environment, content: &Vec<GastNode>) -> ExecutionResult;
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

pub trait ForEachExecutor {
    fn execute(&self,
               env: Environment,
               before: &GastNode,
               body: &GastNode)
               -> ExecutionResult;
}


pub trait GeneratorExecutor {
    fn execute(&self, env: Environment, source: &GastNode, target: &GastNode) -> ExecutionResult;
}

pub trait FilterExecutor {
    fn execute(&self, env: Environment, source: &GastNode, condition: &GastNode) -> ExecutionResult;
}

pub trait MapExecutor {
    fn execute(&self, env: Environment, source: &GastNode, map: &GastNode) -> ExecutionResult;
}

pub trait AndThenExecutor {
    fn execute(&self, env: Environment, first: &GastNode, second: &GastNode) -> ExecutionResult;
}

pub trait CallExecutor {
    fn execute(&self, env: Environment, target: &GastNode, args: &[GastNode], kwargs: &[GastNode]) -> ExecutionResult;
}

pub trait FunctionDefExecutor {
    fn execute(&self, 
               env: Environment, 
               name: &String,
               args: &[GastNode],
               kw_args: &[GastNode],
               vararg: &Option<String>,
               kw_vararg: &Option<String>,
               body: &GastNode) -> ExecutionResult;
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

