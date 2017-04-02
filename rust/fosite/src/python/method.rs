use core::*;

use std::collections::HashMap;

pub struct PythonMethod {

}

impl MethodExecutor for PythonMethod {
    fn execute(&self,
        env: Environment,
        parent: &Pointer,
        address: &Pointer)
        -> Pointer {

        let Environment {vm, ..} = env;
        let parent = parent.clone();
        let address = address.clone();

        let fun = move | env: Environment, mut args: Vec<Mapping>, _: &HashMap<String, GastNode> | {
            let mut new_args = vec!(Mapping::simple(Path::empty(), parent.clone()));
            new_args.append(&mut args);

            let Environment { vm, executors } = env;

            vm.call(executors, &address, new_args).unwrap()
        };

        let pointer = vm.object_of_type(&"method".to_owned());

        vm.set_callable(pointer.clone(), fun);

        return pointer;
    }
}