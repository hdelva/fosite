use core::*;

pub struct PythonGenerator {

}

impl GeneratorExecutor for PythonGenerator {
    fn execute(&self, env: Environment, source: &GastNode, target: &GastNode) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let source_result = vm.execute(executors, source);
        let mut changes = source_result.changes;
        let dependencies = source_result.dependencies;

        let mut mapping = Mapping::new();
        for (path, address) in source_result.result {
            let obj = vm.get_object(&address);

            // todo, replace current node with the node of the generator
            for (el_path, el_address) in obj.get_any_element(vm.current_node()) {
                let mut new_path = path.clone();
                for node in el_path {
                    new_path.add_node(node);
                }

                mapping.add_mapping(new_path, el_address);
            }
        }

        let mapping = mapping.clone()
            .augment(PathNode::Assignment(vm.current_node().clone(), target.to_string()));

        changes.push(AnalysisItem::Identifier(target.to_string()));

        let path = vm.current_path().clone();

        vm.store_identifier_change(AnalysisItem::Identifier(target.to_string()), &path, &mapping);

        {
            let mut scope = vm.last_scope_mut();
            scope.set_mapping(target.to_string(), path, mapping);
        }

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        }
    }
}

pub struct PythonFilter {

}

impl FilterExecutor for PythonFilter {
    fn execute(&self, env: Environment, source: &GastNode, condition: &GastNode) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let source_result = vm.execute(executors, source);
        let mut changes = source_result.changes;
        let mut dependencies = source_result.dependencies;

        let mut condition_result = vm.execute(executors, condition);
        changes.append(&mut condition_result.changes);
        dependencies.append(&mut condition_result.dependencies);

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        } 
    }
}

pub struct PythonMap {

}

impl MapExecutor for PythonMap {
    fn execute(&self, env: Environment, source: &GastNode, op: &GastNode) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let source_result = vm.execute(executors, source);
        let mut changes = source_result.changes;
        let mut dependencies = source_result.dependencies;

        let mut op_result = vm.execute(executors, op);
        changes.append(&mut op_result.changes);
        dependencies.append(&mut op_result.dependencies);

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: op_result.result,
        }      
    }
}

pub struct PythonAndThen {

}

impl AndThenExecutor for PythonAndThen {
    fn execute(&self, env: Environment, first: &GastNode, second: &GastNode) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let first_result = vm.execute(executors, first);
        let mut changes = first_result.changes;
        let mut dependencies = first_result.dependencies;

        let mut second_result = vm.execute(executors, second);
        changes.append(&mut second_result.changes);
        dependencies.append(&mut second_result.dependencies);

        let result_mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

        ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: dependencies,
            changes: changes,
            result: result_mapping,
        }         
    }
}