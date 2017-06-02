use core::*;

pub struct PythonSlice { }

impl SliceExecutor for PythonSlice {
    fn execute(&self, env: Environment, value: &GastNode, lower_node: &GastNode, upper_node: &GastNode) -> ExecutionResult {
        let Environment { mut vm, executors } = env;

        let value_result = vm.execute(executors, value);

        let lower;
        let upper;

        match &lower_node.kind {
            &NodeType::Int {ref value} => {
                lower = value.clone() as i16;
            },
            _ => lower = 0,
        }

        match &upper_node.kind {
            &NodeType::Int {ref value} => {
                if *value < 0 {
                    upper = -1 * value.clone() as i16;
                } else {
                    upper = 0;
                }
            },
            _ => upper = 0,
        }

        let dependencies = value_result.dependencies;
        let changes = value_result.changes;

        let mut result_mapping = Mapping::new();

        for (path, address) in value_result.result.into_iter() {
            let t;
            let elements;
            {
                let o = vm.get_object(&address);
                t = o.get_extension().last().unwrap().clone();
                elements = o.slice_elements(lower, upper);
            }
            
            let n = vm.object_of_type_pointer(&t);
            {
                let o = vm.get_object_mut(&n);
                let mut collection = Collection::new();
                collection.set_content(elements);
                o.set_elements(collection);
            }

            result_mapping.add_mapping(path, n);
        }

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        }
    }
}