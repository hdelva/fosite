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

        let Environment {mut vm, executors} = env;

        let module = vm.retrieve_module(module_name);

        if let Some(module) = module {
            let pointers = module.make_object(vm, parts.clone());

            {
                let mut scope;
                if let &Some(ref into) = into {
                    let ptr = vm.object_of_type(&"module".to_owned());
                    let obj = vm.get_object_mut(&ptr);

                    obj.assign_attribute(into.clone(), Path::empty(), Mapping::simple(Path::empty(), ptr));
                    scope = obj.get_scope_mut();
                } else {
                    scope = vm.last_scope_mut();
                }

                for (name, pointer) in pointers {
                    scope.set_mapping(name.clone(), Path::empty(), Mapping::simple(Path::empty(), pointer));
                }
            }

            vm.insert_module(module_name.clone(), module);
        }

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec!(),
            changes: vec!(),
            result: Mapping::new(),
        };

    }
}