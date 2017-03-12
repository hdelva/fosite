use super::*;

use std::collections::HashSet;
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

    // control flow might keep us into some paths
    // even though the actual code branch is no longer being executed 
    // sometimes we do need to know what the last code branch was
    branches: Vec<GastID>,

    restrictions: Vec<Vec<Path>>,
    watches: Vec<Watch>,

    
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
            branches: vec!(),
            restrictions: Vec::new(),
            watches: Vec::new(),
        }
    }

    pub fn push_branch(&mut self, node: GastID) {
        self.branches.push(node);
    }

    pub fn pop_branch(&mut self) -> Option<GastID> {
        self.branches.pop()
    }

    pub fn current_branch(&self) -> Option<&GastID> {
        self.branches.last()
    }

    pub fn start_watch(&mut self) {
        self.watches.push(Watch::new());
    }

    pub fn toggle_watch(&mut self) {
        self.watches.last_mut().unwrap().toggle();
    }

    pub fn pop_watch(&mut self) -> Watch {
        self.watches.pop().unwrap()
    }

    //
    pub fn notify_change(&mut self, identifier: AnalysisItem, mapping: Mapping) {
        if let Some(watch) = self.watches.last_mut() {
            watch.store(identifier, mapping);
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

    pub fn add_restrictions(&mut self, mut new: Vec<Path>) {
        let mut old = match self.restrictions.last() {
            Some(o) => o.clone(),
            _ => Vec::new(),
        };

        old.append(&mut new);

        self.restrictions.push(old);
    }

    pub fn drop_restrictions(&mut self) {
        self.restrictions.pop();
    }

    pub fn filter(&self, input: ExecutionResult) -> ExecutionResult {
        if self.restrictions.len() == 0 {
            return input;
        }

        let restrictions = self.restrictions.last().unwrap();

        if restrictions.len() == 0 {
            return input;
        }

        let mut new_mapping = Mapping::new();

        'outer:
        for (path, address) in input.result.into_iter() {
            for restriction in restrictions {
                if path.contains(restriction) {
                    continue 'outer;
                }
            }

            new_mapping.add_mapping(path, address);
        }

        return ExecutionResult {
            flow: input.flow,
            changes: input.changes,
            dependencies: input.dependencies,
            result: new_mapping,
        }
    }

    pub fn index(&mut self,
                 executors: &Executors,
                 target: &GastNode,
                 index: &GastNode)
                 -> ExecutionResult {
        match executors.index {
            Some(ref executor) => {
                let env = Environment::new(self, executors);
                executor.execute(env, target, index)
            }
            None => panic!("VM is not setup to execute indexing"),
        }
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

    pub fn boolop(&mut self,
                 executors: &Executors,
                 left: &GastNode,
                 op: &String,
                 right: &GastNode)
                 -> ExecutionResult {
        match executors.boolop {
            Some(ref boolop) => {
                let env = Environment::new(self, executors);
                boolop.execute(env, left, op, right)
            }
            None => panic!("VM is not setup to execute boolean operations"),
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

    pub fn while_loop(&mut self,
                       executors: &Executors,
                       test: &GastNode,
                       body: &GastNode)
                       -> ExecutionResult {
        match executors.while_loop {
            Some(ref while_loop) => {
                let env = Environment::new(self, executors);
                while_loop.execute(env, test, body)
            }
            None => panic!("VM is not setup to execute while loops"),
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

    pub fn list(&mut self, executors: &Executors, content: &Vec<GastNode>) -> ExecutionResult {
        match executors.list {
            Some(ref list) => {
                let env = Environment::new(self, executors);
                list.execute(env, content)
            }
            None => panic!("VM is not setup to execute list literals"),
        }
    }

    pub fn set(&mut self, executors: &Executors, content: &Vec<GastNode>) -> ExecutionResult {
        match executors.set {
            Some(ref set) => {
                let env = Environment::new(self, executors);
                set.execute(env, content)
            }
            None => panic!("VM is not setup to execute set literals"),
        }
    }

    pub fn dict(&mut self, executors: &Executors, content: &Vec<GastNode>) -> ExecutionResult {
        match executors.dict {
            Some(ref dict) => {
                let env = Environment::new(self, executors);
                dict.execute(env, content)
            }
            None => panic!("VM is not setup to execute dictionary literals"),
        }
    }

    pub fn sequence(&mut self, executors: &Executors, content: &Vec<GastNode>) -> ExecutionResult {
        match executors.sequence {
            Some(ref sequence) => {
                let env = Environment::new(self, executors);
                sequence.execute(env, content)
            }
            None => panic!("VM is not setup to execute sequence literals"),
        }
    }

    pub fn load_identifier(&mut self, executors: &Executors, name: &String) -> ExecutionResult {
        match executors.identifier {
            Some(ref identifier) => {
                let result;
                {
                    let env = Environment::new(self, executors);
                    result = identifier.execute(env, name);
                }

                if let Some(watch) = self.watches.last_mut() {
                    // bit dirty, assumes that the relevant dependency is the first one
                    watch.store(result.dependencies.first().unwrap().clone(), result.result.clone());
                }

                return result;
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
                let result;
                {
                    let env = Environment::new(self, executors);
                    result = attribute.execute(env, parent, name);
                }

                if let Some(watch) = self.watches.last_mut() {
                    // bit dirty, assumes that the relevant dependency is the first one
                    watch.store(result.dependencies.first().unwrap().clone(), result.result.clone());
                }

                return result;
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
            None => panic!("VM is not setup to execute assignments"),
        }
    }

    pub fn break_loop(&mut self, executors: &Executors) -> ExecutionResult {
        match executors.break_loop {
            Some(ref break_loop) => {
                let env = Environment::new(self, executors);
                break_loop.execute(env)
            }
            None => panic!("VM is not setup to execute break statements"),
        }
    }

    pub fn continue_loop(&mut self, executors: &Executors) -> ExecutionResult {
        match executors.continue_loop {
            Some(ref continue_loop) => {
                let env = Environment::new(self, executors);
                continue_loop.execute(env)
            }
            None => panic!("VM is not setup to execute continue statements"),
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
            &NodeType::BoolOp { ref left, ref right, ref op, .. } => {
                self.boolop(executors, left, op, right)
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
            &NodeType::While { ref test, ref body } => {
                self.while_loop(executors, test, body)
            }
            &NodeType::Break {  } => {
                self.break_loop(executors)
            }
            &NodeType::Continue {  } => {
                self.continue_loop(executors)
            }
            &NodeType::List { ref content } => {
                self.list(executors, content)
            }
            &NodeType::Set { ref content } => {
                self.set(executors, content)
            }
            &NodeType::Sequence {ref content } => {
                self.sequence(executors, content)
            }
            &NodeType::Index {ref target, ref index} => {
                self.index(executors, target, index)
            }
            &NodeType::Dict {ref content} => {
                self.dict(executors, content)
            }
            _ => panic!("Unsupported Operation\n{:?}", kind),
        };

        let _ = self.nodes.pop();

        let result = self.filter(result);

        return result;
    }

    pub fn change_branch(&mut self, changes: &Vec<AnalysisItem>) {
        let mut identifier_changed = false;

        let set: HashSet<_> = changes.iter().collect(); // dedup
        let changes: Vec<_> = set.into_iter().collect();

        for change in changes {
            if let &AnalysisItem::Object { ref address, .. } = change {
                let mut object = self.memory.get_object_mut(address);
                object.change_branch();
            } else if let &AnalysisItem::Identifier { .. } = change {
                identifier_changed = true;
            }
        }

        if identifier_changed {
            self.scopes.last_mut().unwrap().change_branch();
        }
    }

    pub fn merge_branches(&mut self, changes: &Vec<AnalysisItem>) {
        self.merge_once(changes, None)
    }

    fn merge_once(&mut self, changes: &Vec<AnalysisItem>, cutoff: Option<GastID>) {
        let mut identifier_changed = false;

        let set: HashSet<_> = changes.iter().collect(); // dedup
        let changes: Vec<_> = set.into_iter().collect();

        for change in changes {
            if let &AnalysisItem::Object { ref address, .. } = change {
                let mut object = self.memory.get_object_mut(address);
                object.merge_until(cutoff);
            } else if let &AnalysisItem::Identifier { .. } = change {
                identifier_changed = true;
            }
        }

        if identifier_changed {
            self.scopes.last_mut().unwrap().merge_until(cutoff);
        }
    }

    // merge branches as long as the last node's id is too big
    // if there's no cutoff, collapse a single branch
    pub fn merge_until(&mut self, changes: &Vec<AnalysisItem>, cutoff: Option<GastID>) {
        if let Some(cutoff) = cutoff {
            while let Some(path) = self.paths.pop() {
                let mut id = 0;

                if let Some(node) = path.iter().last() {
                    id = node.get_location()
                } 
                
                if cutoff >= id {
                    self.paths.push(path);
                    break;
                }
            }
        } 

        self.merge_once(changes, cutoff);
    }

    pub fn lift_branches(&mut self, changes: &Vec<AnalysisItem>) {
        let mut identifier_changed = false;

        for change in changes {
            if let &AnalysisItem::Object { ref address, .. } = change {
                let mut object = self.memory.get_object_mut(address);
                object.lift_branches();
            } else if let &AnalysisItem::Identifier { .. } = change {
                identifier_changed = true;
            }
        }

        if identifier_changed {
            self.scopes.last_mut().unwrap().lift_branches();
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
            object.make_type(true);
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
            object.make_type(true);
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

    pub fn is_subtype(&self, type_name1: &String, type_name2: &String) -> bool {
        if type_name1 == type_name2 {
            return true;
        }
        let type_pointer1 = self.knowledge_base.get_type(type_name1);
        return self.is_instance(type_pointer1.unwrap(), type_name2);
    }

    pub fn is_instance(&self, object: &Pointer, type_name: &String) -> bool {
        let type_pointer = self.knowledge_base.get_type(&type_name);
        return self.ancestors(object).contains(type_pointer.unwrap());
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
