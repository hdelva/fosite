use core::*;

use core::Path;
use std::collections::btree_map::Entry;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use std::collections::HashSet;

pub struct PythonWhile { }

impl WhileExecutor for PythonWhile {
    fn execute(&self,
               env: Environment,
               test: &GastNode,
               body: &GastNode)
               -> ExecutionResult {
        let Environment { vm, executors } = env;

        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();

        vm.start_watch();
        let test_result = vm.execute(executors, test);
        vm.toggle_watch();

        let mut no = Vec::new();

        for change in test_result.changes.into_iter() {
            total_changes.push(change);
        }

        for dependency in test_result.dependencies.into_iter() {
            total_dependencies.push(dependency);
        }

        let f = vm.knowledge().constant(&"False".to_owned());

        // split up the test result into yes/no/maybe
        for (path, address) in test_result.result.into_iter() {
            if address == f {
                no.push(path);
            }
        }

        let result = self.branch(vm, executors, body, total_changes, total_dependencies);

        return result;
    }
}

impl PythonWhile {
    fn branch(&self,
              vm: &mut VirtualMachine,
              executors: &Executors,
              body: &GastNode,
              c: Vec<AnalysisItem>,
              d: Vec<AnalysisItem>) -> ExecutionResult {

        let mut total_changes = c;
        let mut total_dependencies = d;


        let mut new_path = vm.current_path().clone();
        new_path.add_node(PathNode::Loop(vm.current_node().clone()));
        vm.push_path(new_path);

        // first iter
        let mut body_result = vm.execute(executors, body);

        total_changes.append(&mut body_result.changes);
        total_dependencies.append(&mut body_result.dependencies);

        vm.pop_path();

        self.check_changes(vm);

        vm.merge_loop(&total_changes);

        return ExecutionResult {
            changes: total_changes,
            dependencies: total_dependencies,
            flow: FlowControl::Continue,
            result: Mapping::new(),
        };
    }

    fn check_changes(&self, vm: &mut VirtualMachine) {
        let watch = vm.pop_watch();

        // initialise an empty path as a problem path
        // will get expanded everytime a variable is unchanged
        let mut problems = vec!(Path::empty());

        // because the initial problem path is empty 
        // it's only an actual problem if things get merged into it
        let mut real_problem = false;

        for (identifier, addresses) in watch.identifiers_before.into_iter() {
            // me too thanks
            let mut same = BTreeSet::new();

            let opt_mapping = watch.identifiers_changed.get(&identifier);

            let identifier_invariants;
            if let Some(new_mapping) = opt_mapping {
                identifier_invariants = possible_identifier_invariants(&addresses, new_mapping);
            } else {
                let mut pls = BTreeMap::new();
                for address in addresses {
                    let mut pls2 = BTreeSet::new();
                    pls2.insert(Path::empty());
                    pls.insert(address, pls2);
                }
                identifier_invariants = pls;
            }

            // quit early when there's something that hasn't changed
            // 
            if identifier_invariants.len() == 0 {
                return;
            }

            for (address, identifier_paths) in identifier_invariants.into_iter() {
                if let Some(object_paths) = watch.objects_changed.get(&address) {
                    // address hasn't changed 
                    // but the object at that address has changed under some conditions 
                    for identifier_path in identifier_paths {
                        let mut invariants = possible_object_invariants(&identifier_path, object_paths);
                        if invariants.len() > 0 {
                            real_problem = true;
                        }
                        same.append(&mut invariants);
                    }
                } else {
                    // address hasn't changed 
                    // object at that address hasn't either
                    real_problem = true;
                    for identifier_path in identifier_paths {
                        same.insert(identifier_path);
                    }
                }
            }

            let mut new_problems = Vec::new();

            for problem in problems.iter() {
                for new_problem in same.iter() {
                    if problem.mergeable(new_problem) {
                        let mut new = problem.clone();
                        new.merge_into(new_problem.clone());
                        new_problems.push(new);
                    }
                }
            }

            problems = new_problems;
        }

        if real_problem {
            let content = WhileLoopChange::new(problems);
            let message = Message::Output {
                source: vm.current_node().clone(),
                content: Box::new(content),
            };
            &CHANNEL.publish(message);
        }
    }
}

fn possible_identifier_invariants(old: &HashSet<Pointer>, changes: &Mapping) -> BTreeMap<Pointer, BTreeSet<Path>> {
    let mut relevant_reversals = BTreeMap::new();
    let mut all_changes = BTreeSet::new();
    let mut all_reversals = BTreeSet::new();

    for &(ref path, _) in changes.iter() {
        all_changes.insert(path.clone());
    }

    for &(ref path, _) in changes.iter(){
        let change_reversals = path.reverse();

        for reversal in change_reversals.into_iter().rev() {
            all_reversals.insert(reversal);
        }
    }

    for &(ref path, ref address) in changes.iter(){
        let mut change_reversals = path.reverse();

        if old.contains(address) {
            change_reversals.push(path.clone());
        }

        'outer:
        for reversal in change_reversals.into_iter().rev() {
            for existing in all_reversals.iter() {
                if existing.contains(&reversal) && !reversal.contains(existing) {
                    continue 'outer;
                } 
            }

            for change in &all_changes {
                if reversal.contains(change) {
                    continue 'outer;
                } 
            }

            match relevant_reversals.entry(*address) {
                Entry::Vacant(v) => {
                    let mut acc = BTreeSet::new();
                    acc.insert(reversal.clone());
                    v.insert(acc);
                }
                Entry::Occupied(mut o) => {
                    
                    o.get_mut().insert(reversal.clone());
                }
            };    
        }
    }

    return relevant_reversals;
}

fn possible_object_invariants(parent_path: &Path, changes: &Vec<Path>) -> BTreeSet<Path> {
    // remove obsolete entries first, i.e.
    // (1, 5) and (1, 5, 9)
    let mut all_reversals = BTreeSet::new();

    for change in changes {
        let change_reversals = change.reverse();
        for reversal in change_reversals.iter().rev() {
            all_reversals.insert(reversal.clone());
        }
    }

    let mut possibilities = BTreeSet::new();
    'outer:
    for reversal in all_reversals.iter() {
        for change in changes {
            if reversal.contains(change) {
                continue 'outer;
            }
        }

        for existing in all_reversals.iter() {
            if existing.contains(reversal) && !reversal.contains(existing) {
                continue 'outer;
            } 
        }

        if parent_path.mergeable(reversal) {
            let mut new_path = parent_path.clone();
            new_path.merge_into(reversal.clone());
            possibilities.insert(new_path);
        }
    }

    return possibilities;
}