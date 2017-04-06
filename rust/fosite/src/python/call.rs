use core::*;

pub struct PythonCall {

}

impl CallExecutor for PythonCall {
    fn execute(&self, 
               env: Environment, 
               target: &GastNode, 
               arg_nodes: &[GastNode], 
               kwarg_nodes: &[GastNode]) -> ExecutionResult {
        let Environment { vm, executors } = env;

        let mut total_changes: Vec<AnalysisItem> = Vec::new();
        let mut total_dependencies = Vec::new();
        let mut mapping = Mapping::new();

        // evaluate the arguments first
        let mut args: Vec<Mapping> = Vec::new();
        let mut kwargs: Vec<(String, Mapping)> = Vec::new();

        for arg in arg_nodes.iter() {
            let mut arg_result = vm.execute(executors, arg);
            total_changes.append(&mut arg_result.changes);
            total_dependencies.append(&mut arg_result.dependencies);
            args.push(arg_result.result);
        }

        for kwarg in kwarg_nodes.iter() {
            if let &NodeType::Argument{ref name, ref value} = &kwarg.kind {
                let mut kwarg_result = vm.execute(executors, value);
                total_changes.append(&mut kwarg_result.changes);
                total_dependencies.append(&mut kwarg_result.dependencies);
                kwargs.push( (name.clone(), kwarg_result.result) );
            }
        }

        let mut target_result = vm.execute(executors, target);
        total_changes.append(&mut target_result.changes);
        total_dependencies.append(&mut target_result.dependencies);

        let len = target_result.result.len();

        // collect all the paths
        // will be used to zip with the call results later
        let mut paths = Vec::new();

        // keep these separated from the argument results
        let mut body_changes = Vec::new();
        let mut body_dependencies = Vec::new();

        for (index, (path, address)) in target_result.result.into_iter().enumerate() {
            let new_node = PathNode::Frame(
                    vm.current_node().clone(), 
                    Some(target.to_string()), 
                    index as i16, 
                    len as i16);

            let mut aug_args = Vec::new();
            let mut aug_kwargs = Vec::new();

            for a in args.iter() {
                aug_args.push(a.clone().augment(new_node.clone()));
            }

            for &(ref n, ref a) in kwargs.iter() {
                aug_kwargs.push( (n.clone(), a.clone().augment(new_node.clone())) );
            }

            // collect all the paths
            // will be used to zip with the call results later
            paths.push(path);
                
            // very important when an object's content is changed 
            // in multiple possible calls
            // this will hide one possible call from the other
            let mut current_path = vm.current_path().clone();
            current_path.add_node( 
                PathNode::Frame(
                    vm.current_node().clone(), 
                    Some(target.to_string()), 
                    index as i16, 
                    len as i16));

            vm.push_path(current_path);

            // todo filter the body changes
            if let Some(body_result) = vm.call(executors, &address, aug_args, aug_kwargs) {
                for change in body_result.changes.into_iter() {
                    if let &AnalysisItem::Object(_) = &change {
                        body_changes.push(change);
                    }
                }
                
                for dependency in body_result.dependencies.into_iter() {
                    if let &AnalysisItem::Object(_) = &dependency {
                        body_changes.push(dependency);
                    }
                }
            }

            let _ = vm.pop_path();
            
            vm.next_branch(&body_changes);
        }

        vm.merge_function(&body_changes);

        // combine all the analysis results
        // only transfer object changes
        total_changes.append(&mut body_changes);
        total_dependencies.append(&mut body_dependencies);

        // the VM has the results of the function calls
        let results: Vec<OptionalMapping> = vm.get_results();

        for (index, (opt_mapping, target_path)) in results.into_iter().zip(paths).enumerate() {
            for (result_path, opt_address) in opt_mapping.into_iter() {
                // combine both paths
                let mut total_path = result_path;
                total_path.merge_into(target_path.clone());

                // add the glue between the two paths
                total_path.add_node( 
                    PathNode::Frame(
                        vm.current_node().clone(), 
                        Some(target.to_string()), 
                        index as i16, 
                        len as i16));

                if let Some(address) = opt_address {
                    mapping.add_mapping(total_path, address);
                } else {
                    // warning, path didn't return
                }
            }
        }

        return ExecutionResult {
            changes: total_changes,
            dependencies: total_dependencies,
            flow: FlowControl::Continue,
            result: mapping,
        };        
    }
}