use super::Pointer;
use super::Scope;
use super::Path;
use super::Mapping;

pub struct Context {
    private_scope: Scope,
    protected_scope: Scope,
    public_scope: Scope,
    context_type: Option<Pointer>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            private_scope: Scope::new(),
            protected_scope: Scope::new(),
            public_scope: Scope::new(),
            context_type: None,
        }
    }

    pub fn assign_public(&mut self, name: String, path: Path, mapping: Mapping) {
        self.public_scope.set_mapping(name, path, mapping);
    }

    pub fn get_public_scope(&self) -> &Scope {
        &self.public_scope
    }

    pub fn get_public_scope_mut(&mut self) -> &mut Scope {
        &mut self.public_scope
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ExecutionContext {
    context: Context,
    hidden: Scope,
    result: Vec<Pointer>, // collection of possible result values at the end of execution
}
