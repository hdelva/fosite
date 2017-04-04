use core::*;
use std::sync::Mutex;

lazy_static! {
    static ref ARGS: Mutex<Vec<Vec<(String, Mapping)>>> = Mutex::new(Vec::new());
    static ref KW_ARGS: Mutex<Vec<Vec<(String, Mapping)>>> = Mutex::new(Vec::new());
    static ref VARARG: Mutex<Vec<Option<String>>> = Mutex::new(Vec::new());
    static ref KW_VARARG: Mutex<Vec<Option<String>>> = Mutex::new(Vec::new());
    static ref BODY: Mutex<Vec<GastNode>> = Mutex::new(Vec::new());
}

    


pub struct PythonFunction {

}

impl FunctionDefExecutor for PythonFunction {
    fn execute(&self, 
               env: Environment, 
               name: &String,
               rpos: &[GastNode],
               rkw: &[GastNode],
               vararg: &Option<String>,
               kw_vararg: &Option<String>,
               body: &GastNode) -> ExecutionResult {

        let Environment {vm, executors} = env;

        let mut dependencies = Vec::new();
        let mut changes = Vec::new();

        let mut rpos_evaluated = Vec::new();
        let mut rkw_evaluated = Vec::new();

        for node in rpos.iter() {
            if let &NodeType::Argument {ref name, ref value} = &node.kind {
                let mut eval_result = vm.execute(executors, value);
                rpos_evaluated.push((name.clone(), eval_result.result));
                dependencies.append(&mut eval_result.dependencies);
                changes.append(&mut eval_result.changes);
            }
        }

        for node in rkw.iter() {
            if let &NodeType::Argument {ref name, ref value} = &node.kind {
                let mut eval_result = vm.execute(executors, value);
                rkw_evaluated.push((name.clone(), eval_result.result));
                dependencies.append(&mut eval_result.dependencies);
                changes.append(&mut eval_result.changes);
            }
        }

        let index;
        unsafe {
            index = ARGS.lock().unwrap().len();

            ARGS.lock().unwrap().push(rpos_evaluated);
            KW_ARGS.lock().unwrap().push(rkw_evaluated);
            VARARG.lock().unwrap().push(vararg.clone());
            KW_VARARG.lock().unwrap().push(kw_vararg.clone());
            BODY.lock().unwrap().push(body.clone());
        }
        

        let inner = move | env: Environment, args: Vec<Mapping>, kw_args: Vec<(String, Mapping)> | {
            let Environment { vm, executors } = env;

            let mut body_result;
            unsafe {
                assign_positional(vm, executors,
                         &ARGS.lock().unwrap()[index], &KW_ARGS.lock().unwrap()[index], 
                         &args, &kw_args,
                         &VARARG.lock().unwrap()[index], &KW_VARARG.lock().unwrap()[index]);

                body_result = vm.execute(executors, &BODY.lock().unwrap()[index]);
            }

            
            let execution_result = ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: body_result.dependencies,
                changes: body_result.changes,
                result: Mapping::new(),
            };

            execution_result
        };

        let pointer = vm.object_of_type(&"function".to_owned());

        vm.set_callable(pointer.clone(), inner);
        vm.assign_direct(executors, name.clone(), Mapping::simple(Path::empty(), pointer));

        return ExecutionResult {
            flow: FlowControl::Continue,
            changes: vec!(),
            dependencies: vec!(),
            result: Mapping::new(),
        }
    }
}

fn assign_positional(vm: &mut VirtualMachine,
                        executors: &Executors,
                        rpos: &[(String, Mapping)],
                        rkw: &[(String, Mapping)],
                        gpos: &[Mapping],
                        gkw: &[(String, Mapping)],
                        vararg: &Option<String>,
                        kw_vararg: &Option<String>) 
                        -> ExecutionResult {

    let mut dependencies = vec!();
    let mut changes = vec!();

    if rpos.len() > 0 && gpos.len() > 0 {
        let &(ref name, ref default) = &rpos[0];
        let mapping = gpos[0].clone();
        vm.assign_direct(executors, name.clone(), mapping);
        let mut intermediate = assign_positional(vm, executors, &rpos[1..], rkw, &gpos[1..], gkw, vararg, kw_vararg);
        dependencies.append(&mut intermediate.dependencies);
        changes.append(&mut intermediate.changes);
    } else {
        let mut intermediate = assign_vararg(vm, executors, rkw, gpos, gkw, vararg, kw_vararg);
        dependencies.append(&mut intermediate.dependencies);
        changes.append(&mut intermediate.changes);
    }

    return ExecutionResult {
        flow: FlowControl::Continue,
        dependencies: dependencies,
        changes: changes,
        result: Mapping::new(),
    }
}

fn assign_vararg(vm: &mut VirtualMachine,
                        executors: &Executors,
                        rkw: &[(String, Mapping)],
                        gpos: &[Mapping],
                        gkw: &[(String, Mapping)],
                        vararg: &Option<String>,
                        kw_vararg: &Option<String>) 
                        -> ExecutionResult {

    let mut dependencies = vec!();
    let mut changes = vec!();

    if vararg.is_some() {
        //todo assign rest of gpos
        let mut intermediate = assign_kw_positional(vm, executors, rkw, &[], gkw, kw_vararg);
        dependencies.append(&mut intermediate.dependencies);
        changes.append(&mut intermediate.changes);
    } else {
        let mut intermediate = assign_kw_positional(vm, executors, rkw, gpos, gkw, kw_vararg);
        dependencies.append(&mut intermediate.dependencies);
        changes.append(&mut intermediate.changes);
    }

    return ExecutionResult {
        flow: FlowControl::Continue,
        dependencies: dependencies,
        changes: changes,
        result: Mapping::new(),
    }
}

fn assign_kw_positional(vm: &mut VirtualMachine,
                        executors: &Executors,
                        rkw: &[(String, Mapping)],
                        gpos: &[Mapping],
                        gkw: &[(String, Mapping)],
                        kw_vararg: &Option<String>) 
                        -> ExecutionResult {

    let mut dependencies = vec!();
    let mut changes = vec!();

    if rkw.len() > 0 && gpos.len() > 0 {
        let &(ref name, ref default) = &rkw[0];
        let mapping = gpos[0].clone();
        vm.assign_direct(executors, name.clone(), mapping);
        let mut intermediate = assign_kw_positional(vm, executors, &rkw[1..], &gpos[1..], gkw, kw_vararg);
        dependencies.append(&mut intermediate.dependencies);
        changes.append(&mut intermediate.changes);
    } else {
        let mut intermediate = assign_kw(vm, executors, rkw, gkw, kw_vararg);
        dependencies.append(&mut intermediate.dependencies);
        changes.append(&mut intermediate.changes);
    }

    return ExecutionResult {
        flow: FlowControl::Continue,
        dependencies: dependencies,
        changes: changes,
        result: Mapping::new(),
    }
}

fn assign_kw(vm: &mut VirtualMachine,
                        executors: &Executors,
                        rkw: &[(String, Mapping)],
                        gkw: &[(String, Mapping)],
                        kw_vararg: &Option<String>) 
                        -> ExecutionResult {

    let mut dependencies = vec!();
    let mut changes = vec!();

    if rkw.len() > 0 {
        let &(ref name, ref default) = &rkw[0];

        let mut mapping = default.clone();
        let mut next = Vec::new();

        for &(ref given_name, ref given_mapping) in gkw.iter() {
            if name == given_name {
                mapping = given_mapping.clone();
            } else {
                next.push((given_name.clone(), given_mapping.clone()));
            }
        }

        vm.assign_direct(executors, name.clone(), mapping);
        let mut intermediate = assign_kw(vm, executors, &rkw[1..], &next, kw_vararg);
        dependencies.append(&mut intermediate.dependencies);
        changes.append(&mut intermediate.changes);
    } else {
        let mut intermediate = assign_kw_vararg(vm, executors, gkw, kw_vararg);
        dependencies.append(&mut intermediate.dependencies);
        changes.append(&mut intermediate.changes);
    }

    return ExecutionResult {
        flow: FlowControl::Continue,
        dependencies: dependencies,
        changes: changes,
        result: Mapping::new(),
    }
}

fn assign_kw_vararg(vm: &mut VirtualMachine,
                        executors: &Executors,
                        gkw: &[(String, Mapping)],
                        kw_vararg: &Option<String>) 
                        -> ExecutionResult {

    let mut dependencies = vec!();
    let mut changes = vec!();

    if kw_vararg.is_some() {
        //todo assign rest of gkw
        
    }

    return ExecutionResult {
        flow: FlowControl::Continue,
        dependencies: dependencies,
        changes: changes,
        result: Mapping::new(),
    }
}                 
