use super::*;

use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::BTreeSet;
use std::iter::FromIterator;
use std::slice::Iter;

pub struct VirtualMachine {
    // todo call stack
    scopes: Vec<Scope>,
    pub memory: Memory, // todo make private
    knowledge_base: KnowledgeBase,

    // todo, can we use a single Path
    // would require a way to shrink Paths
    paths: Vec<Path>,
    nodes: Vec<GastID>,
}

impl VirtualMachine {
    pub fn new() -> VirtualMachine {
        let memory = Memory::new();
        let knowledge = KnowledgeBase::new();
        VirtualMachine {
            scopes: Vec::new(),
            memory: memory,
            knowledge_base: knowledge,
            nodes: vec![],
            paths: vec![Path::empty()],
        }
    }

    pub fn scopes(&self) -> Iter<Scope> {
        return self.scopes.iter();
    }

    pub fn last_scope_mut(&mut self) -> &mut Scope {
        return self.scopes.last_mut().unwrap();
    }

    pub fn pop_path(&mut self) -> Path {
        self.paths.pop().unwrap()
    }

    pub fn push_path(&mut self, path: Path) {
        self.paths.push(path);
    }

    pub fn current_path(&self) -> &Path {
        self.paths.last().unwrap()
    }

    pub fn binop(&mut self,
                 executors: &Executors,
                 left: &GastNode,
                 op: &String,
                 right: &GastNode)
                 -> ExecutionResult {
        match executors.binop {
            Some(ref binop) => {
                let env = Environment::new(self, executors);
                binop.execute(env, left, op, right)
            }
            None => panic!("VM is not setup to execute binary operations"),
        }
    }

    pub fn conditional(&mut self,
                       executors: &Executors,
                       test: &GastNode,
                       body: &GastNode,
                       or_else: &GastNode)
                       -> ExecutionResult {
        match executors.conditional {
            Some(ref conditional) => {
                let env = Environment::new(self, executors);
                conditional.execute(env, test, body, or_else)
            }
            None => panic!("VM is not setup to execute conditionals"),
        }
    }

    pub fn block(&mut self, executors: &Executors, content: &Vec<GastNode>) -> ExecutionResult {
        match executors.block {
            Some(ref block) => {
                let env = Environment::new(self, executors);
                block.execute(env, content)
            }
            None => panic!("VM is not setup to execute blocks"),
        }
    }

    pub fn load_identifier(&mut self, executors: &Executors, name: &String) -> ExecutionResult {
        match executors.identifier {
            Some(ref identifier) => {
                let env = Environment::new(self, executors);
                identifier.execute(env, name)
            }
            None => panic!("VM is not setup to execute identifiers"),
        }
    }

    pub fn load_attribute(&mut self,
                          executors: &Executors,
                          parent: &GastNode,
                          name: &String)
                          -> ExecutionResult {
        match executors.attribute {
            Some(ref attribute) => {
                let env = Environment::new(self, executors);
                attribute.execute(env, parent, name)
            }
            None => panic!("VM is not setup to execute attributes"),
        }
    }

    pub fn boolean(&mut self, executors: &Executors, value: bool) -> ExecutionResult {
        match executors.boolean {
            Some(ref boolean) => {
                let env = Environment::new(self, executors);
                boolean.execute(env, value)
            }
            None => panic!("VM is not setup to execute booleans"),
        }
    }

    pub fn string(&mut self, executors: &Executors) -> ExecutionResult {
        match executors.string {
            Some(ref string) => {
                let env = Environment::new(self, executors);
                string.execute(env)
            }
            None => panic!("VM is not setup to execute strings"),
        }
    }

    pub fn int(&mut self, executors: &Executors) -> ExecutionResult {
        match executors.int {
            Some(ref int) => {
                let env = Environment::new(self, executors);
                int.execute(env)
            }
            None => panic!("VM is not setup to execute integers"),
        }
    }

    pub fn float(&mut self, executors: &Executors) -> ExecutionResult {
        match executors.float {
            Some(ref float) => {
                let env = Environment::new(self, executors);
                float.execute(env)
            }
            None => panic!("VM is not setup to execute floats"),
        }
    }

    pub fn declaration(&mut self,
                       executors: &Executors,
                       name: &String,
                       kind: &String)
                       -> ExecutionResult {
        match executors.declaration {
            Some(ref declaration) => {
                let env = Environment::new(self, executors);
                declaration.execute(env, name, kind)
            }
            None => panic!("VM is not setup to execute declarations"),
        }
    }

    pub fn assign(&mut self,
                  executors: &Executors,
                  targets: &Vec<GastNode>,
                  value: &GastNode)
                  -> ExecutionResult {
        match executors.assign {
            Some(ref assign) => {
                let env = Environment::new(self, executors);
                assign.execute(env, targets, value)
            }
            None => panic!("VM is not setup to execute declarations"),
        }
    }

    pub fn execute(&mut self, executors: &Executors, node: &GastNode) -> ExecutionResult {
        let ref id = node.id;
        let ref kind = node.kind;

        self.nodes.push(id.clone());

        let result = match kind {
            &NodeType::Boolean { ref value } => self.boolean(executors, *value),
            &NodeType::String { .. } => self.string(executors),
            &NodeType::Int { .. } => self.int(executors),
            &NodeType::Float { .. } => self.float(executors),
            &NodeType::Nil {} => self.load_identifier(executors, &"None".to_owned()),
            &NodeType::BinOp { ref left, ref right, ref op, .. } => {
                self.binop(executors, left, op, right)
            }
            &NodeType::If { ref test, ref body, ref or_else } => {
                self.conditional(executors, test, body, or_else)
            }
            &NodeType::Block { ref content } => self.block(executors, content),
            &NodeType::Identifier { ref name } => self.load_identifier(executors, name),
            &NodeType::Attribute { ref parent, ref attribute } => {
                self.load_attribute(executors, parent, attribute)
            }
            &NodeType::Declaration { ref id, ref kind } => self.declaration(executors, id, kind),
            &NodeType::Assignment { ref targets, ref value } => {
                self.assign(executors, targets, value)
            }
            _ => panic!("Unsupported Operation"),
        };

        {
            let mut items = HashMap::new();
            items.insert("node".to_owned(),
                         MessageItem::String(format!("{:?}", result)));
            let message = Message::Notification {
                source: id.clone(),
                kind: NPROCESSED_NODE,
                content: items,
            };
            &CHANNEL.publish(message);
        }

        let _ = self.nodes.pop();

        return result;
    }

    pub fn change_branch(&mut self, identifier_changed: bool, changes: &HashSet<AnalysisItem>) {
        if identifier_changed {
            self.scopes.last_mut().unwrap().change_branch();
        }

        for change in changes {
            if let &AnalysisItem::Object { ref address } = change {
                let mut object = self.memory.get_object_mut(address);
                object.change_branch();
            }
        }
    }

    pub fn merge_branches(&mut self, identifier_changed: bool, changes: &HashSet<AnalysisItem>) {
        if identifier_changed {
            self.scopes.last_mut().unwrap().merge_branches();
        }

        for change in changes {
            if let &AnalysisItem::Object { ref address } = change {
                let mut object = self.memory.get_object_mut(address);
                object.merge_branches();
            }
        }
    }

    pub fn object_of_type(&mut self, type_name: &String) -> Pointer {
        let pointer = self.memory.new_object();
        let object = self.memory.get_object_mut(&pointer);
        let type_pointer = self.knowledge_base.get_type(&type_name);

        match type_pointer {
            Some(address) => {
                object.extend(address.clone());
            }
            _ => panic!("There is no type with name {}", type_name),
        }

        return pointer;
    }

    // todo, implement more generic
    pub fn declare_new_constant(&mut self, name: &String, tpe: &String) -> ExecutionResult {
        let pointer = self.object_of_type(tpe);
        let mut scope = self.scopes.last_mut().unwrap();
        let mapping = Mapping::simple(Path::empty(), pointer);
        scope.set_constant(name.clone(),
                           self.paths.last().unwrap().clone(),
                           mapping.clone());
        self.knowledge_base.add_constant(name, &pointer);
        let result = Mapping::simple(Path::empty(), self.knowledge_base.constant("None"));
        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![AnalysisItem::Identifier { name: name.clone() }],
            result: result,
        };
    }

    // todo, implement more generic
    pub fn declare_simple_type(&mut self, name: &String) {
        let pointer = self.memory.new_object();
        {
            let mut object = self.memory.get_object_mut(&pointer);
            object.set_type(true);
        }
        let mapping = Mapping::simple(Path::empty(), pointer);
        let mut scope = self.scopes.last_mut().unwrap();
        scope.set_mapping(name.clone(),
                          self.paths.last().unwrap().clone(),
                          mapping.clone());
        self.knowledge_base.add_type(name.clone(), pointer.clone());
    }

    pub fn declare_sub_type(&mut self, executors: &Executors, name: &String, parent: &String) {
        let result = self.load_identifier(executors, parent).result;
        let (_, parent_pointer) = result.iter().next().unwrap();

        let new_pointer = self.memory.new_object();
        {
            let mut object = self.memory.get_object_mut(&new_pointer);
            object.set_type(true);
            object.extend(parent_pointer.clone());
        }

        let mapping = Mapping::simple(Path::empty(), new_pointer.clone());
        let mut scope = self.scopes.last_mut().unwrap();
        scope.set_mapping(name.clone(),
                          self.paths.last().unwrap().clone(),
                          mapping.clone());

        self.knowledge_base.add_type(name.clone(), new_pointer.clone());
    }

    pub fn knowledge_base(&mut self) -> &mut KnowledgeBase {
        return &mut self.knowledge_base;
    }

    pub fn new_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn get_object(&self, address: &Pointer) -> &Object {
        self.memory.get_object(address)
    }

    pub fn get_object_mut(&mut self, address: &Pointer) -> &mut Object {
        self.memory.get_object_mut(address)
    }

    pub fn ancestors(&self, pointer: &Pointer) -> Vec<Pointer> {
        let object = self.memory.get_object(pointer);

        let mut result = Vec::new();

        let types = object.get_extension();

        for tpe in types {
            result.push(tpe.clone());
            let mut intermediate = self.ancestors(tpe).clone();
            result.append(&mut intermediate);
        }

        return result;
    }

    pub fn common_ancestor(&self, first: &Pointer, second: &Pointer) -> BTreeSet<Pointer> {
        let first_ancestors: BTreeSet<_> = BTreeSet::from_iter(self.ancestors(first).into_iter());
        let second_ancestors: BTreeSet<_> = BTreeSet::from_iter(self.ancestors(second).into_iter());

        let intersection = &first_ancestors & &second_ancestors;
        return intersection;
    }

    pub fn current_node(&self) -> GastID {
        self.nodes.last().unwrap_or(&0).clone()
    }

    pub fn knowledge(&self) -> &KnowledgeBase {
        &self.knowledge_base
    }

    pub fn knowledge_mut(&mut self) -> &mut KnowledgeBase {
        &mut self.knowledge_base
    }
}
