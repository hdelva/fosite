use core::*;

pub struct PythonConditional { }

impl ConditionalExecutor for PythonConditional {
    fn execute(&self,
               env: Environment,
               test: &GastNode,
               body: &GastNode,
               or_else: &GastNode)
               -> ExecutionResult {
        let Environment { vm, executors } = env;

        let mut total_changes = Vec::new();
        let mut total_dependencies = Vec::new();

        let test_result = vm.execute(executors, test);

        let mut no = Vec::new();
        let mut yes = Vec::new();

        for change in test_result.changes.into_iter() {
            total_changes.push(change);
        }

        for dependency in test_result.dependencies.into_iter() {
            total_dependencies.push(dependency);
        }

        //println!("??????changes {:?}", total_changes);
        //println!("??????dependencies {:?}", total_dependencies);

        let t = vm.knowledge().constant(&"True".to_owned());
        let f = vm.knowledge().constant(&"False".to_owned());

        // split up the test result into yes/no/maybe
        for (path, address) in test_result.result.into_iter() {
            if address == t {
                yes.push(path);
            } else if address == f {
                no.push(path);
            }
        }

        self.branch(vm, executors, body, or_else, yes, no, total_changes, total_dependencies)
    }
}

impl PythonConditional {
    fn branch(&self,
              vm: &mut VirtualMachine,
              executors: &Executors,
              body: &GastNode,
              or_else: &GastNode,
              yes: Vec<Path>,
              no: Vec<Path>,
              c: Vec<AnalysisItem>,
              d: Vec<AnalysisItem>) -> ExecutionResult {
        let original_restriction = vm.get_branch_restrictions().clone();

        let mut total_changes = c;
        let mut total_dependencies = d;

        let mut positive;
        let mut negative;
        {
            let current_path = vm.current_path();

            positive = current_path.clone();
            positive.add_node(PathNode::Condition(vm.current_node().clone(), 0, 2));
            negative = current_path.clone();
            negative.add_node(PathNode::Condition(vm.current_node().clone(), 1, 2));
        }

        vm.push_path(positive);
        vm.add_branch_restrictions(no.clone());
        let body_result = vm.execute(executors, body);
        vm.pop_path();
        vm.set_branch_restrictions(original_restriction.clone());

        let changes = body_result.changes;
        let dependencies = body_result.dependencies;

        for change in &changes {
            total_changes.push(change.clone());
        }

        for dependency in &dependencies {
            total_dependencies.push(dependency.clone());
        }

        vm.next_branch(&total_changes);

        vm.push_path(negative);
        vm.add_branch_restrictions(yes.clone());
        let else_result = vm.execute(executors, or_else);
        vm.pop_path();
        vm.set_branch_restrictions(original_restriction.clone());

        let changes = else_result.changes;
        let dependencies = else_result.dependencies;

        for change in &changes {
            total_changes.push(change.clone());
        }

        for dependency in &dependencies {
            total_dependencies.push(dependency.clone());
        }

        let mut hide_as_loop = Vec::new();
        let flow;
        
        // lawd jezus why
        match (body_result.flow, else_result.flow) {
            (FlowControl::Continue, FlowControl::Continue) => {
                flow = FlowControl::Continue;
                hide_as_loop.push(None);
                hide_as_loop.push(None);
            }
            (FlowControl::Continue, FlowControl::TerminateCall) => {
                flow = FlowControl::Continue;
                hide_as_loop.push(None);
                hide_as_loop.push(Some(false));
            }
            (FlowControl::Continue, FlowControl::TerminateLoop) => {
                flow = FlowControl::Continue;
                hide_as_loop.push(None);
                hide_as_loop.push(Some(true));
            }
            (FlowControl::TerminateCall, FlowControl::Continue) => {
                flow = FlowControl::Continue;
                hide_as_loop.push(Some(false));
                hide_as_loop.push(None);
            }
            (FlowControl::TerminateLoop, FlowControl::Continue) => {
                flow = FlowControl::Continue;
                hide_as_loop.push(Some(true));
                hide_as_loop.push(None);
            }
            (FlowControl::TerminateCall, FlowControl::TerminateCall) => {
                hide_as_loop.push(Some(false));
                hide_as_loop.push(Some(false));
                flow = FlowControl::TerminateCall;
            },
            (FlowControl::TerminateLoop, FlowControl::TerminateCall) => {
                hide_as_loop.push(Some(true));
                hide_as_loop.push(Some(false));
                flow = FlowControl::TerminateLoop;
            },
            (FlowControl::TerminateCall, FlowControl::TerminateLoop) => {
                hide_as_loop.push(Some(false));
                hide_as_loop.push(Some(true));
                flow = FlowControl::TerminateLoop;
            },
            (FlowControl::TerminateLoop, FlowControl::TerminateLoop) => {
                hide_as_loop.push(Some(false));
                hide_as_loop.push(Some(false));
                flow = FlowControl::TerminateLoop;
            },
        }

        vm.merge_branches(&total_changes, hide_as_loop, vec!(no, yes));

        ExecutionResult {
            changes: total_changes,
            dependencies: total_dependencies,
            flow: flow,
            result: Mapping::new(),
        }
    }
}
