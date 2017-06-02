use core::*;

pub struct PythonImport {

}

impl ImportExecutor for PythonImport {
    fn execute(&self,
               env: Environment,
               module_name: &String,
               parts: &Vec<(String, String)>,
               into: &Option<String>)
               -> ExecutionResult {

        let Environment {mut vm, ..} = env;

        let module = vm.retrieve_module(module_name);

        if let Some(module) = module {
            let pointers = module.make_object(vm, parts.clone());

            {
                let mut scope;
                let path = vm.current_path().clone();
                if let &Some(ref into) = into {
                    // where should this thing go
                    let mut ptr = -1;
                    if let Some(existing) = vm.knowledge().get_type(into) {
                        // `into` is a typename, we're defining class attributes
                        ptr = existing.clone();
                    } 
                    
                    if ptr < 0 {
                        // create a new module of this name
                        ptr = vm.object_of_type(&"module".to_owned());
                    }

                    {
                        let mut vm_scope = vm.last_scope_mut();
                        vm_scope.set_mapping(into.clone(), path.clone(), Mapping::simple(Path::empty(), ptr.clone()));
                    }

                    let obj = vm.get_object_mut(&ptr);
                    scope = obj.get_scope_mut();
                } else {
                    scope = vm.last_scope_mut();
                }

                for (name, pointer) in pointers {
                    scope.set_mapping(name.clone(), path.clone(), Mapping::simple(Path::empty(), pointer));
                }
            }

            vm.insert_module(module_name.clone(), module);
        } 

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec!(),
            changes: vec!(),
            result: Mapping::new(),
        }
    }
}