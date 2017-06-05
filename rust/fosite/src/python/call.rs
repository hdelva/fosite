use core::*;

use std::collections::BTreeSet;
use std::cmp;


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
        let mut result_mapping = Mapping::new();

        // evaluate the arguments first
        let mut args: Vec<Mapping> = Vec::new();
        let mut kwargs: Vec<(String, Mapping)> = Vec::new();

        let mut results: Vec<Vec<(Path, Mapping)>> = Vec::new();

        let mut node = vm.current_node().clone();
        node.pop();
        vm.add_node(node);

        for arg in arg_nodes.iter() {
            let mut arg_result = vm.execute(executors, arg);
            total_changes.append(&mut arg_result.changes);
            total_dependencies.append(&mut arg_result.dependencies);
            args.push(arg_result.result);
        }

        for kwarg in kwarg_nodes.iter() {
            if let NodeType::Argument{ref name, ref value} = kwarg.kind {
                let mut kwarg_result = vm.execute(executors, value);
                total_changes.append(&mut kwarg_result.changes);
                total_dependencies.append(&mut kwarg_result.dependencies);
                kwargs.push( (name.clone(), kwarg_result.result) );
            }
        }

        vm.pop_node();

        //println!("------changes {:?}", total_changes);
        //println!("------dependencies {:?}", total_dependencies);

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

            for a in &args {
                aug_args.push(a.clone().augment(new_node.clone()));
            }

            for &(ref n, ref a) in &kwargs {
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

            vm.new_result();

            // todo filter the body changes
            if let Some(body_result) = vm.call(executors, &address, aug_args, aug_kwargs) {
                for change in body_result.changes {
                    if let AnalysisItem::Object(_) = change {
                        body_changes.push(change);
                    }
                }
                
                for dependency in body_result.dependencies {
                    if let AnalysisItem::Object(_) = dependency {
                        body_changes.push(dependency);
                    }
                }
            }

            results.push(vm.get_result());

            vm.pop_path();
            
            vm.next_branch(&body_changes);
        }

        vm.merge_function(&body_changes);

        // combine all the analysis results
        // only transfer object changes
        total_changes.append(&mut body_changes);
        total_dependencies.append(&mut body_dependencies);

        let mut good = Vec::new();
        for temp in &results {
            for &(ref p, _) in temp.iter() {
                good.push(p.prune(vm.current_node()));
            }
        }

        let bad = bad_paths(good);
        if !bad.is_empty() {
            let content = NoReturn::new(bad);
            let message = Message::Output {
                source: vm.current_node().clone(),
                content: Box::new(content),
            };
            CHANNEL.publish(message);
        }

        for (index, (return_points, target_path)) in results.into_iter().zip(paths).enumerate() {
            let mut pls = Mapping::new();

            for (p1, m) in return_points {
                for (mut p2, a) in m {
                    p2.merge_into(p1.clone());
                    pls.add_mapping(p2, a);
                }
            }

            if pls.is_empty() {
                pls.add_mapping(Path::empty(), vm.knowledge().constant("None"));
            }

            for &(ref result_path, ref address) in &pls {
                // combine both paths
                let mut total_path = result_path.clone();
                total_path.merge_into(target_path.clone());

                // add the glue between the two paths
                total_path.add_node( 
                    PathNode::Frame(
                        vm.current_node().clone(), 
                        Some(target.to_string()), 
                        index as i16, 
                        len as i16));

                result_mapping.add_mapping(total_path, *address);
            }
        }

        ExecutionResult {
            changes: total_changes,
            dependencies: total_dependencies,
            flow: FlowControl::Continue,
            result: result_mapping,
        }        
    }
}

fn get_branch_points(original: Vec<Path>) -> BTreeSet<Vec<PathNode>> {
    let mut result = BTreeSet::new();
    for og in original {
        let mut pls = Vec::new();
        for node in og {
            if node.is_branch() {
                pls.push(node);
            }
        }
        result.insert(pls);
    }

    result
}

fn get_other_paths(original: &[PathNode]) -> BTreeSet<Vec<PathNode>> {
    let mut result = BTreeSet::new();
    let head = &original[..original.len() - 1];
    let tail = &original[original.len() - 1];

    for opposite in tail.reverse() {
        let mut new = head.to_vec();
        new.push(opposite);
        result.insert(new);
    }

    result
}

fn bad_paths(original: Vec<Path>) -> BTreeSet<Path> {
    let mut branches = get_branch_points(original);

    let mut cleaned = BTreeSet::new();
    'outer:
    for path in &branches {
        for other_path in &branches {
            let l = cmp::min(path.len(), other_path.len());
            if path.len() > other_path.len() && other_path[..l] == path[..l] {
                continue 'outer;
            }
        }

        cleaned.insert(path.clone());
    }
    branches = cleaned;
    
    let mut todo = BTreeSet::new();
    
    // init
    for b in &branches {
        if !b.is_empty() {
            let mut temp = get_other_paths(b);
            todo.append(&mut temp);
        } else {
            // shit's fine, unconditional return
            return BTreeSet::new();
        }
    }

    let mut cleaned = BTreeSet::new();
    'outer2:
    for path in &todo {
        for other_path in &todo {
            let l = cmp::min(path.len(), other_path.len());
            if path.len() < other_path.len() && other_path[..l] == path[..l] {
                continue 'outer2;
            }
        }

        cleaned.insert(path.clone());
    }
    todo = cleaned;

    let mut done = false;
    while !done {
        done = true;

        let mut next = BTreeSet::new();
        for path in todo {
            if branches.contains(&path) {
                done = false;
                if path.len() == 1 {
                    // shit's fine, both cases return
                    return BTreeSet::new();
                } else {
                    let mut temp = get_other_paths(&path[..path.len()-1]);
                    next.append(&mut temp);
                }
                branches.insert(path[..path.len()-1].to_vec());
            } else {
                next.insert(path);
            }
        }

        todo = next;
    }

    let mut merged = BTreeSet::new();
    for path in &todo {
        let mut m = false;
        for other_path in &todo {
            if mergeable_paths(path, other_path) {
                let mut new_path = path.clone();
                new_path.append(&mut other_path.clone());
                merged.insert(new_path);
                m = true;
            }
        }

        if !m {
            merged.insert(path.clone());
        }
    }

    todo = merged;

    let mut result = BTreeSet::new();

    for nodes in todo {
        let mut path = Path::empty();
        for node in nodes {
            path.add_node(node);
        }
        result.insert(path);
    }

    result
}

fn mergeable_paths(path: &[PathNode], other_path: &[PathNode]) -> bool {
    for n1 in path.iter() {
        for n2 in other_path.iter() {
            if n1.get_location() == n2.get_location() {
                return false;
            }
        }
    }

    true
}