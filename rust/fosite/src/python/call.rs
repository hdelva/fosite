use core::*;

use std::collections::HashMap;

pub struct PythonCall {

}

impl CallExecutor for PythonCall {
    fn execute(&self, env: Environment, target: &GastNode, args: &Vec<GastNode>) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let mut total_changes: Vec<AnalysisItem> = Vec::new();
        let mut total_dependencies = Vec::new();
        let mut mapping = Mapping::new();

        let mut target_result = vm.execute(executors, target);

        let len = target_result.result.len();

        for (index, (path, address)) in target_result.result.into_iter().enumerate() {
                
            let mut current_path = vm.current_path().clone();
            current_path.add_node( 
                PathNode::Frame(
                    vm.current_node().clone(), 
                    Some(target.to_string()), 
                    index as i16, 
                    len as i16));

            vm.push_path(current_path);

            if let Some(mut body_result) = vm.call(executors, &address, args) {
                // todo, have the merge step return all the values
                for (result_path, result_address) in body_result.result.into_iter() {
                    let mut new_path = result_path.clone();
                    new_path.merge_into(path.clone());
                    mapping.add_mapping(result_path, result_address);
                }

                total_changes.append(&mut body_result.changes);
                total_dependencies.append(&mut body_result.dependencies);
            }

            let _ = vm.pop_path();
            
            vm.next_branch(&total_changes);
        }

        // todo, don't merge; discard
        vm.merge_branches(&total_changes);

        // append these at the end
        // these changes shouldn't influence the merging of the frame nodes
        total_changes.append(&mut target_result.changes);
        total_dependencies.append(&mut target_result.dependencies);

        return ExecutionResult {
            changes: total_changes,
            dependencies: total_dependencies,
            flow: FlowControl::Continue,
            result: mapping,
        };        
    }
}