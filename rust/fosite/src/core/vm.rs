use super::*;

use std::collections::HashSet;
use std::collections::BTreeSet;
use std::iter::FromIterator;
use std::slice::Iter;
use std::collections::HashMap;

type Callable = Fn(Environment, Vec<Mapping>, Vec<(String, Mapping)>) -> ExecutionResult;

pub struct VirtualMachine {
    scopes: Vec<Scope>,
    shadow_scopes: Vec<Scope>,
    pub memory: Memory, // todo make private
    knowledge_base: KnowledgeBase,

    // todo, can we use a single Path
    // would require a way to shrink Paths
    paths: Vec<Path>,
    nodes: Vec<PathID>,
    default: PathID,

    watches: Vec<Watch>,

    branch_restrictions: Vec<Vec<Path>>,

    // calls
    callables: HashMap<Pointer, Box<Callable>>,
    closures: HashMap<Pointer, Scope>,
    results: Vec<Vec<(Path, Mapping)>>,

    // modules 
    modules: HashMap<String, Module>,
}

impl VirtualMachine {
    pub fn new() -> VirtualMachine {
        let mut path = Path::empty();
        path.add_node(PathNode::Frame(vec!(0), None, 0, 1));

        let memory = Memory::new();
        let knowledge = KnowledgeBase::new();
        VirtualMachine {
            scopes: Vec::new(),
            shadow_scopes: Vec::new(),
            memory: memory,
            knowledge_base: knowledge,
            nodes: vec![vec!(0)],
            paths: vec![path],
            branch_restrictions: Vec::new(),
            watches: Vec::new(),
            default: vec!(0),
            callables: HashMap::new(),
            closures: HashMap::new(),
            results: Vec::new(),
            modules: HashMap::new(),
        }
    }

    pub fn retrieve_module(&mut self, name: &String) -> Option<Module> {
        self.modules.remove(name)
    }

    pub fn insert_module(&mut self, name: String, module: Module) {
        self.modules.insert(name, module);
    }

    pub fn define_function<T: 'static>(&mut self, name: String, callable: T) where
        T : for<'r> Fn(Environment<'r>, Vec<Mapping>, Vec<(String, Mapping)>) -> ExecutionResult {
        let pointer = self.object_of_type(&"function".to_owned());
        self.set_callable(pointer, callable);

        let mapping = Mapping::simple(Path::empty(), pointer);

        let path = self.current_path().clone();
        let mut scope = self.scopes.last_mut().unwrap();
        scope.set_mapping(name, path, mapping);
    }

    pub fn define_method<T: 'static>(&mut self, tpe: String, name: String, callable: T) where
        T : for<'r> Fn(Environment<'r>, Vec<Mapping>, Vec<(String, Mapping)>) -> ExecutionResult {
        let pointer = self.object_of_type(&"method".to_owned());
        self.set_callable(pointer, callable);

        let parent_ptr = self.knowledge().get_type(&tpe).unwrap().clone();

        let mapping = Mapping::simple(Path::empty(), pointer);

        let path = self.current_path().clone();
        let mut object = self.get_object_mut(&parent_ptr);
        object.assign_attribute(name, path, mapping);
    }

    pub fn is_callable(&self, address: &Pointer) -> bool {
        self.callables.contains_key(address)
    }

    pub fn set_callable<T: 'static>(&mut self, address: Pointer, callable: T) where
        T : for<'r> Fn(Environment<'r>, Vec<Mapping>, Vec<(String, Mapping)>) -> ExecutionResult {
        self.callables.insert(address, Box::new(callable));
    }

    pub fn set_closure(&mut self, address: Pointer, scope: Scope) {
        self.closures.insert(address, scope);
    }

    pub fn get_result(&mut self) -> Vec<(Path, Mapping)> {
        self.results.pop().unwrap()
    }

    pub fn add_result(&mut self, path: Path, mapping: Mapping) {
        if let Some(r) = self.results.last_mut() {
            r.push((path, mapping));
        }
        
    }

    pub fn new_result(&mut self) {
        self.results.push(Vec::new());
    }

    pub fn call(&mut self, 
                executors: &Executors, 
                address: &Pointer, 
                args: Vec<Mapping>, 
                kwargs: Vec<(String, Mapping)>) 
                -> Option<ExecutionResult> {

        let b = self.scopes.len() > 2;

        // move the current function scope to the shadows
        if b {
            let s1 = self.scopes.pop().unwrap();
            let s2 = self.scopes.pop().unwrap();
            self.shadow_scopes.push(s2);
            self.shadow_scopes.push(s1);
        }

        // take ownership of the closure scope
        let closure = self.closures.remove(address);
        if let Some(closure) = closure {
            self.scopes.push(closure);
        } else {
            self.scopes.push(Scope::new());
        }
        
        // local scope
        self.scopes.push(Scope::new());

        let analysis;
        // take ownership of the callable
        if let Some(callable) = self.callables.remove(address) {
            {
                let env = Environment {vm: self, executors: executors};
                analysis = Some(callable(env, args, kwargs));
            }
            // give the callable back back to the VM
            self.callables.insert(address.clone(), callable);
        } else {
            analysis = None;
        }

        // remove the function scope
        let _ = self.scopes.pop();

        // remove the enclosing scope
        let closure = self.scopes.pop().unwrap();

        // put the closure back into the VM 
        self.closures.insert(address.clone(), closure);

        // restore the previous function scopes if necessary
        if b {
            let s1 = self.shadow_scopes.pop().unwrap();
            let s2 = self.shadow_scopes.pop().unwrap();

            self.scopes.push(s2);
            self.scopes.push(s1);
        }

        // return the dependency information
        analysis
    }

    pub fn start_watch(&mut self) {
        let node = self.current_node().clone();
        self.watches.push(Watch::new(node));
    }

    pub fn toggle_watch(&mut self) {
        self.watches.last_mut().unwrap().toggle();
    }

    pub fn pop_watch(&mut self) -> Watch {
        self.watches.pop().unwrap()
    }

    pub fn store_identifier_dependency(&mut self, identifier: AnalysisItem, mapping: &Mapping) {
        if let Some(watch) = self.watches.last_mut() {
            watch.store_identifier_dependency(identifier, mapping);
        }
    }

    pub fn store_object_dependency(&mut self, address: Pointer) {
        if let Some(watch) = self.watches.last_mut() {
            watch.store_object_dependency(address);
        }
    }

    pub fn store_identifier_change(&mut self, identifier: AnalysisItem, path: &Path, mapping: &Mapping) {
        if let Some(watch) = self.watches.last_mut() {
            watch.store_identifier_change(identifier, path, mapping);
        }
    }

    pub fn store_object_change(&mut self, pointer: Pointer, path: &Path) {
        if let Some(watch) = self.watches.last_mut() {
            watch.store_object_change(pointer, path);
        }
    }

    pub fn scopes(&self) -> Iter<Scope> {
        self.scopes.iter()
    }

    pub fn last_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    pub fn pop_path(&mut self) {
        self.paths.pop();
    }

    pub fn push_path(&mut self, path: Path) {
        self.paths.push(path);
    }

    pub fn current_path(&self) -> &Path {
        self.paths.last().unwrap()
    }

    pub fn add_branch_restrictions(&mut self, new: Vec<Path>) {
        self.branch_restrictions.push(new);
    }

    pub fn get_branch_restrictions(&self) -> &Vec<Vec<Path>> {
        &self.branch_restrictions
    }

    pub fn set_branch_restrictions(&mut self, restrictions: Vec<Vec<Path>>) {
        self.branch_restrictions = restrictions;
    }

    pub fn filter(&mut self, input: ExecutionResult) -> ExecutionResult {
        if self.branch_restrictions.len() == 0 {
            return input;
        }

        let mut new_mapping = Mapping::new();

        'outer:
        for (path, address) in input.result.into_iter() {
            for restriction_set in self.branch_restrictions.iter() {
                for restriction in restriction_set.iter() {
                    if path.contains(restriction) {
                        continue 'outer;
                    }
                }   
            }

            new_mapping.add_mapping(path, address);
        }

        ExecutionResult {
            flow: input.flow,
            changes: input.changes,
            dependencies: input.dependencies,
            result: new_mapping,
        }
    }

    pub fn make_method_object(&mut self, executors: &Executors, parent: &Pointer, address: &Pointer) -> Pointer {
        match executors.method {
            Some(ref method) => {
                let env = Environment::new(self, executors);
                method.execute(env, parent, address)
            }
            None => panic!("VM is not setup to create method objects"),
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

    pub fn function(&mut self,
                 executors: &Executors,
                 name: &String,
                 args: &[GastNode],
                 kw_args: &[GastNode],
                 vararg: &Option<String>,
                 kw_vararg: &Option<String>,
                 body: &GastNode) -> ExecutionResult {
        match executors.function {
            Some(ref executor) => {
                let env = Environment::new(self, executors);
                executor.execute(env, name, args, kw_args, vararg, kw_vararg, body)
            }
            None => panic!("VM is not setup to execute function definitions"),
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

    pub fn foreach(&mut self,
                       executors: &Executors,
                       before: &GastNode,
                       body: &GastNode)
                       -> ExecutionResult {
        match executors.foreach {
            Some(ref foreach) => {
                let env = Environment::new(self, executors);
                foreach.execute(env, before, body)
            }
            None => panic!("VM is not setup to execute foreach loops"),
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

    pub fn negate(&mut self, executors: &Executors, content: &GastNode) -> ExecutionResult {
        match executors.negate {
            Some(ref negate) => {
                let env = Environment::new(self, executors);
                negate.execute(env, content)
            }
            None => panic!("VM is not setup to execute negations"),
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

    pub fn import(&mut self,
                       executors: &Executors,
                       module: &String,
                       parts: &Vec<(String, String)>,
                       into: &Option<String>)
                       -> ExecutionResult {
        match executors.import {
            Some(ref import) => {
                let env = Environment::new(self, executors);
                import.execute(env, module, parts,  into)
            }
            None => panic!("VM is not setup to execute imports"),
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

    pub fn assign_direct(&mut self,
                  executors: &Executors,
                  target: String,
                  value: Mapping)
                  -> ExecutionResult {
        match executors.assign {
            Some(ref assign) => {
                let env = Environment::new(self, executors);
                assign.direct(env, target, value)
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

    pub fn _call(&mut self, executors: &Executors, target: &GastNode, args: &[GastNode], kwargs: &[GastNode]) -> ExecutionResult {
        match executors.call {
            Some(ref call) => {
                let env = Environment::new(self, executors);
                call.execute(env, target, args, kwargs)
            }
            None => panic!("VM is not setup to execute function calls"),
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

    pub fn unop(&mut self, executors: &Executors, value: &GastNode) -> ExecutionResult {
        match executors.unop {
            Some(ref unop) => {
                let env = Environment::new(self, executors);
                unop.execute(env, value)
            }
            None => panic!("VM is not setup to execute unary operations"),
        }
    }

    pub fn generator(&mut self, executors: &Executors, source: &GastNode, target: &GastNode) -> ExecutionResult {
        match executors.generator {
            Some(ref generator) => {
                let env = Environment::new(self, executors);
                generator.execute(env, source, target)
            }
            None => panic!("VM is not setup to execute generators"),
        }
    }

    pub fn filter_generator(&mut self, executors: &Executors, source: &GastNode, condition: &GastNode) -> ExecutionResult {
        match executors.filter {
            Some(ref filter) => {
                let env = Environment::new(self, executors);
                filter.execute(env, source, condition)
            }
            None => panic!("VM is not setup to filter generators"),
        }
    }

    pub fn map(&mut self, executors: &Executors, source: &GastNode, op: &GastNode) -> ExecutionResult {
        match executors.map {
            Some(ref map) => {
                let env = Environment::new(self, executors);
                map.execute(env, source, op)
            }
            None => panic!("VM is not setup to map generators"),
        }
    }

    pub fn andthen(&mut self, executors: &Executors, first: &GastNode, second: &GastNode) -> ExecutionResult {
        match executors.andthen {
            Some(ref andthen) => {
                let env = Environment::new(self, executors);
                andthen.execute(env, first, second)
            }
            None => panic!("VM is not setup to combine generators"),
        }
    }

    pub fn slice(&mut self, executors: &Executors, target: &GastNode, lower: &GastNode, upper: &GastNode) -> ExecutionResult {
        match executors.slice {
            Some(ref slice) => {
                let env = Environment::new(self, executors);
                slice.execute(env, target, lower, upper)
            }
            None => panic!("VM is not setup to execute slices"),
        }
    }

    pub fn ret(&mut self, executors: &Executors, value: &GastNode) -> ExecutionResult {
        match executors.ret {
            Some(ref ret) => {
                let env = Environment::new(self, executors);
                ret.execute(env, value)
            }
            None => panic!("VM is not setup to return function results"),
        }
    }

    pub fn execute(&mut self, executors: &Executors, node: &GastNode) -> ExecutionResult {
        let ref id = node.id;
        let ref kind = node.kind;

        let mut current = self.nodes.pop().unwrap();
        current.push(id.clone());
        self.nodes.push(current);

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
            &NodeType::Generator {ref source, ref target} => {
                self.generator(executors, source, target)
            }
            &NodeType::Filter {ref source, ref condition} => {
                self.filter_generator(executors, source, condition)
            }
            &NodeType::Map {ref source, ref op} => {
                self.map(executors, source, op)
            }
            &NodeType::AndThen {ref first, ref second} => {
                self.andthen(executors, first, second)
            }
            &NodeType::ForEach {ref before, ref body} => {
                self.foreach(executors, before, body)
            }
            &NodeType::Call {ref target, ref args, ref kwargs} => {
                self._call(executors, target, args, kwargs)
            }
            &NodeType::Import {ref module, ref parts, ref into} => {
                self.import(executors, module, parts, into)
            }
            &NodeType::Negate {ref value} => {
                self.negate(executors, value)
            }
            &NodeType::UnOp {ref value, ..} => {
                self.unop(executors, value)
            }
            &NodeType::Slice {ref target, ref lower, ref upper} => {
                self.slice(executors, target, lower, upper)
            }
            &NodeType::FunctionDef {ref name, ref body, ref args, ref kw_args, ref vararg, ref kw_vararg} => {
                self.function(executors, name, args, kw_args, vararg, kw_vararg, body)
            }
            &NodeType::Return {ref value} => {
                self.ret(executors, value)
            }
            _ => panic!("Unsupported Operation\n{:?}", kind),
        };

        let mut current = self.nodes.pop().unwrap();
        let _ = current.pop();

        self.nodes.push(current);

        self.filter(result)
    }

    pub fn next_branch(&mut self, changes: &Vec<AnalysisItem>) {
        let mut identifier_changed = false;

        let set: HashSet<_> = changes.iter().collect(); // dedup
        let changes: Vec<_> = set.into_iter().collect();

        for change in changes {
            if let &AnalysisItem::Object(ref address) = change {
                let mut object = self.memory.get_object_mut(address);
                object.next_branch();
            } else if let &AnalysisItem::Identifier( _ ) = change {
                identifier_changed = true;
            }
        }

        if identifier_changed {
            self.scopes.last_mut().unwrap().next_branch();
        }
    }

    pub fn reset_branch_counter(&mut self, changes: &Vec<AnalysisItem>) {
        let mut identifier_changed = false;

        let set: HashSet<_> = changes.iter().collect(); // dedup
        let changes: Vec<_> = set.into_iter().collect();

        for change in changes {
            if let &AnalysisItem::Object(ref address) = change {
                let mut object = self.memory.get_object_mut(address);
                object.reset_branch_counter();
            } else if let &AnalysisItem::Identifier( _ ) = change {
                identifier_changed = true;
            }
        }

        if identifier_changed {
            self.scopes.last_mut().unwrap().reset_branch_counter();
        }
    }

    pub fn merge_branches(&mut self, changes: &Vec<AnalysisItem>, hide_as_loop: Vec<Option<bool>>, restrictions: Vec<Vec<Path>>) {
        let mut identifier_changed = false;

        let set: HashSet<_> = changes.iter().collect(); // dedup
        let changes: Vec<_> = set.into_iter().collect();

        for change in changes {
            if let &AnalysisItem::Object (ref address) = change {
                let mut object = self.memory.get_object_mut(address);
                object.merge_branches(hide_as_loop.clone(), &restrictions);
            } else if let &AnalysisItem::Identifier ( _ ) = change {
                identifier_changed = true;
            }
        }

        if identifier_changed {
            self.scopes.last_mut().unwrap().merge_branches(hide_as_loop.clone(), &restrictions);
        }
    }

    pub fn merge_loop(&mut self, changes: &Vec<AnalysisItem>) {
        let mut identifier_changed = false;

        let set: HashSet<_> = changes.iter().collect(); // dedup
        let changes: Vec<_> = set.into_iter().collect();

        for change in changes {
            if let &AnalysisItem::Object (ref address) = change {
                let mut object = self.memory.get_object_mut(address);
                object.merge_loop();
            } else if let &AnalysisItem::Identifier ( _ ) = change {
                identifier_changed = true;
            }
        }

        if identifier_changed {
            self.scopes.last_mut().unwrap().merge_loop();
        }
    }

    pub fn merge_function(&mut self, changes: &Vec<AnalysisItem>) {
        let mut identifier_changed = false;
        
        let set: HashSet<_> = changes.iter().collect(); // dedup
        let changes: Vec<_> = set.into_iter().collect();

        for change in changes {
            if let &AnalysisItem::Object (ref address) = change {
                let mut object = self.memory.get_object_mut(address);
                object.merge_function();
            } else if let &AnalysisItem::Identifier ( _ ) = change {
                identifier_changed = true;
            }
        }

        if identifier_changed {
            self.scopes.pop().unwrap().discard_function();
        }

        //return self.scopes.pop().unwrap().discard_function();
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

        pointer
    }

    pub fn object_of_type_pointer(&mut self, type_pointer: &Pointer) -> Pointer {
        let pointer = self.memory.new_object();
        let object = self.memory.get_object_mut(&pointer);
        object.extend(*type_pointer);

        pointer
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
        
        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec![],
            changes: vec![AnalysisItem::Identifier (name.clone())],
            result: result,
        }
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
        self.knowledge_base.add_type(name.clone(), pointer);
    }

    pub fn declare_sub_type(&mut self, executors: &Executors, name: &String, parent: &String) {
        let result = self.load_identifier(executors, parent).result;
        let &(_, ref parent_pointer) = result.iter().next().unwrap();

        let new_pointer = self.memory.new_object();
        {
            let mut object = self.memory.get_object_mut(&new_pointer);
            object.make_type(true);
            object.extend(*parent_pointer);
        }

        let mapping = Mapping::simple(Path::empty(), new_pointer);
        let mut scope = self.scopes.last_mut().unwrap();
        scope.set_mapping(name.clone(),
                          self.paths.last().unwrap().clone(),
                          mapping.clone());

        self.knowledge_base.add_type(name.clone(), new_pointer);
    }

    pub fn knowledge_base(&mut self) -> &mut KnowledgeBase {
        &mut self.knowledge_base
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
        self.is_instance(type_pointer1.unwrap(), type_name2)
    }

    pub fn is_instance(&self, object: &Pointer, type_name: &String) -> bool {
        let type_pointer = self.knowledge_base.get_type(&type_name);
        
        self.ancestors(object).contains(type_pointer.unwrap())
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

        result
    }

    pub fn common_ancestor(&self, first: &Pointer, second: &Pointer) -> BTreeSet<Pointer> {
        let first_ancestors: BTreeSet<_> = BTreeSet::from_iter(self.ancestors(first).into_iter());
        let second_ancestors: BTreeSet<_> = BTreeSet::from_iter(self.ancestors(second).into_iter());

        &first_ancestors & &second_ancestors
    }

    pub fn current_node(&self) -> &PathID {
        self.nodes.last().unwrap_or(&self.default)
    }

    pub fn add_node(&mut self, node: PathID) {
        self.nodes.push(node);
    }

    pub fn pop_node(&mut self) {
        self.nodes.pop();
    }

    pub fn knowledge(&self) -> &KnowledgeBase {
        &self.knowledge_base
    }

    pub fn knowledge_mut(&mut self) -> &mut KnowledgeBase {
        &mut self.knowledge_base
    }
}
