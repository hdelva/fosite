use core::*;

use std::collections::HashMap;

pub struct PythonMethod {

}

impl MethodExecutor for PythonMethod {
    fn execute(&self,
        env: Environment,
        parent: &Pointer,
        address: &Pointer)
        -> ExecutionResult {

        let Environment {vm, executors} = env;
        let parent = parent.clone();
        let address = address.clone();

        let fun = move | env: Environment, args: &[GastNode], _: &HashMap<String, GastNode> | {
            let node = GastNode::new(0, NodeType::Identifier {name: "___self".to_owned()});
            let mut new_args = vec!(node);
            new_args.extend_from_slice(args);

            let Environment { vm, executors } = env;

            vm.call(executors, &address, &new_args).unwrap()
        };


        let pointer = vm.memory.new_object();
        vm.set_callable(pointer.clone(), fun);

        let mapping = Mapping::simple(Path::empty(), pointer.clone());
        let path = vm.current_path().clone();

        return ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: vec!(),
            changes: vec!(),
            result: mapping,
        };
    }
}
