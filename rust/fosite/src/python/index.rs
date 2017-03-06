use core::*;

pub struct PythonIndex {

}

impl IndexExecutor for PythonIndex {
    fn execute(&self,
               env: Environment,
               target: &GastNode,
               index: &GastNode)
               -> ExecutionResult{
        let Environment { vm, executors } = env;

        // needed to add this indexation to the path
        let current_node = vm.current_node().clone();

        // mapping to return
        let mut result_mapping = Mapping::new();

        let target_result = vm.execute(executors, target);

        // update dependencies and changes
        let mut total_dependencies = Vec::new();
        let mut total_changes = Vec::new();

        let mut dependencies = target_result.dependencies;
        let mut changes = target_result.changes;
        total_dependencies.append(&mut dependencies);
        total_changes.append(&mut changes);

        let index_result = vm.execute(executors, index);
        let mut dependencies = index_result.dependencies;
        let mut changes = index_result.changes;
        total_dependencies.append(&mut dependencies);
        total_changes.append(&mut changes);

        // target object b here
        let target_mapping = target_result.result;

        for (target_path, target_address) in target_mapping.iter() {
            // we obviously depend on the target object
            total_dependencies.push(AnalysisItem::Object { address: target_address.clone(), path: None });

            let target_object = vm.get_object(target_address);
            
            let value_mappings;
            // getting a fixed value can be done more accurately
            match &index.kind {
                &NodeType::Int {ref value} => {
                    value_mappings = target_object.get_element(*value as i16, &current_node);
                },
                _ => {
                    value_mappings = target_object.get_any_element(&current_node);
                }
            }
            
            // add all the possible values to the result map
            for (path, address) in value_mappings.iter() {
                // is it always mergeable?
                if target_path.mergeable(&path) {
                    let mut new_path = target_path.clone();

                    // todo can probably remove this clone
                    new_path.merge_into(path.clone());
                    result_mapping.add_mapping(new_path, *address);
                }
            }
        }
        
        let execution_result = ExecutionResult {
            flow: FlowControl::Continue,
            dependencies: total_dependencies,
            changes: total_changes,
            result: result_mapping,
        };

        return execution_result;
            
    }
}