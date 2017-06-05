use core::*;
use std::sync::Mutex;
use std::collections::BTreeSet;

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
               name: &str,
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
            if let NodeType::Argument {ref name, ref value} = node.kind {
                let mut eval_result = vm.execute(executors, value);
                rpos_evaluated.push((name.clone(), eval_result.result));
                dependencies.append(&mut eval_result.dependencies);
                changes.append(&mut eval_result.changes);
            }
        }

        for node in rkw.iter() {
            if let NodeType::Argument {ref name, ref value} = node.kind {
                let mut eval_result = vm.execute(executors, value);
                rkw_evaluated.push((name.clone(), eval_result.result));
                dependencies.append(&mut eval_result.dependencies);
                changes.append(&mut eval_result.changes);
            }
        }

        let index = ARGS.lock().unwrap().len();

        ARGS.lock().unwrap().push(rpos_evaluated);
        KW_ARGS.lock().unwrap().push(rkw_evaluated);
        VARARG.lock().unwrap().push(vararg.clone());
        KW_VARARG.lock().unwrap().push(kw_vararg.clone());
        BODY.lock().unwrap().push(body.clone());
        

        let inner = move | env: Environment, args: Vec<Mapping>, kw_args: Vec<(String, Mapping)> | {
            let Environment { vm, executors } = env;

            let new_node = vm.current_path()._iter().last().unwrap().clone(); // should be the function call node

            let mut aug_args = Vec::new();
            let mut aug_kwargs = Vec::new();

            for &(ref n, ref a) in &ARGS.lock().unwrap()[index] {
                aug_args.push( (n.clone(), a.clone().augment(new_node.clone())) );
            }

            for &(ref n, ref a) in &KW_ARGS.lock().unwrap()[index] {
                aug_kwargs.push( (n.clone(), a.clone().augment(new_node.clone())) );
            }

            assign_positional(vm, executors,
                        &aug_args, &aug_kwargs, 
                        &args, &kw_args,
                        &VARARG.lock().unwrap()[index], &KW_VARARG.lock().unwrap()[index]);

            let body = &BODY.lock().unwrap()[index].clone();
            let body_result = vm.execute(executors, body);
            
            ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: body_result.dependencies,
                changes: body_result.changes,
                result: Mapping::new(),
            }
        };

        let pointer = vm.object_of_type(&"function".to_owned());

        vm.set_callable(pointer, inner);
        let mut aresult = vm.assign_direct(executors, name.to_owned(), Mapping::simple(Path::empty(), pointer));
        changes.append(&mut aresult.changes);
        dependencies.append(&mut aresult.dependencies);

        ExecutionResult {
            flow: FlowControl::Continue,
            changes: changes,
            dependencies: dependencies,
            result: Mapping::new(),
        }
    }
}

fn assign_positional(vm: &mut VirtualMachine,
                        executors: &Executors,
                        arg: &[(String, Mapping)],
                        kwonly: &[(String, Mapping)],
                        gpos: &[Mapping],
                        gkw: &[(String, Mapping)],
                        vararg: &Option<String>,
                        kw_vararg: &Option<String>) 
                        -> ExecutionResult {

    let mut dependencies = vec!();
    let mut changes = vec!();

    // todo, use the default value before moving to varargs
    if !arg.is_empty() && !gpos.is_empty() {
        let &(ref name, _) = &arg[0];
        let mapping = gpos[0].clone();
        let mut aresult = vm.assign_direct(executors, name.clone(), mapping);
        dependencies.append(&mut aresult.dependencies);
        changes.append(&mut aresult.changes);
        let mut intermediate = assign_positional(vm, executors, &arg[1..], kwonly, &gpos[1..], gkw, vararg, kw_vararg);
        dependencies.append(&mut intermediate.dependencies);
        changes.append(&mut intermediate.changes);
    } else {
        let mut intermediate = assign_vararg(vm, executors, arg, kwonly, gpos, gkw, vararg, kw_vararg);
        dependencies.append(&mut intermediate.dependencies);
        changes.append(&mut intermediate.changes);
    }

    ExecutionResult {
        flow: FlowControl::Continue,
        dependencies: dependencies,
        changes: changes,
        result: Mapping::new(),
    }
}

fn assign_vararg(vm: &mut VirtualMachine,
                        executors: &Executors,
                        arg: &[(String, Mapping)],
                        kwonly: &[(String, Mapping)],
                        gpos: &[Mapping],
                        gkw: &[(String, Mapping)],
                        vararg: &Option<String>,
                        kw_vararg: &Option<String>) 
                        -> ExecutionResult {

    let mut dependencies = vec!();
    let mut changes = vec!();

    if let Some(ref name) = *vararg {
        let type_name = "list".to_owned();
        let obj_ptr = vm.object_of_type(&type_name);        

        let mut chunks = Vec::new();
        for arg in gpos {
            let mut chunk = CollectionChunk::empty();

            for &(ref path, ref address) in arg {
                let kind = vm.get_object(address).get_extension().first().unwrap();
                let repr = Representant::new(*address, *kind, Some(1), Some(1));
                chunk.add_representant(path.clone(), repr);    
            }
            
            chunks.push(chunk);
        }

        {
            let mut obj = vm.get_object_mut(&obj_ptr);
            obj.define_elements(chunks, Path::empty());
        }

        let mapping = Mapping::simple(vm.current_path().clone(), obj_ptr);
        let mut aresult = vm.assign_direct(executors, name.clone(), mapping);
        dependencies.append(&mut aresult.dependencies);
        changes.append(&mut aresult.changes);
    } 

    //todo assign rest of gpos
    let mut intermediate = assign_kw(vm, executors, &[arg, kwonly].concat(), gkw, kw_vararg);
    dependencies.append(&mut intermediate.dependencies);
    changes.append(&mut intermediate.changes);


    ExecutionResult {
        flow: FlowControl::Continue,
        dependencies: dependencies,
        changes: changes,
        result: Mapping::new(),
    }
}

fn assign_kw(vm: &mut VirtualMachine,
                        executors: &Executors,
                        arg: &[(String, Mapping)],
                        gkw: &[(String, Mapping)],
                        kw_vararg: &Option<String>) 
                        -> ExecutionResult {

    let mut dependencies = vec!();
    let mut changes = vec!();

    let mut s1 = BTreeSet::new();
    for &(ref name, _) in arg.iter() {
        s1.insert(name);
    }

    if !gkw.is_empty() && !s1.is_empty() {
        let mut next = Vec::new();

        for &(ref name, ref mapping) in gkw.iter() {
            if s1.contains(name) {
                let mut aresult = vm.assign_direct(executors, name.clone(), mapping.clone());
                dependencies.append(&mut aresult.dependencies);
                changes.append(&mut aresult.changes);
            } else {
                next.push((name.clone(), mapping.clone()));
            }
        }

        let mut intermediate = assign_kw_vararg(vm, executors, &next, gkw, kw_vararg);
        dependencies.append(&mut intermediate.dependencies);
        changes.append(&mut intermediate.changes);
    } else {
        let mut intermediate = assign_kw_vararg(vm, executors, arg, gkw, kw_vararg);
        dependencies.append(&mut intermediate.dependencies);
        changes.append(&mut intermediate.changes);
    }

    ExecutionResult {
        flow: FlowControl::Continue,
        dependencies: dependencies,
        changes: changes,
        result: Mapping::new(),
    }
}

fn assign_kw_vararg(vm: &mut VirtualMachine,
                        executors: &Executors,
                        arg: &[(String, Mapping)],
                        gkw: &[(String, Mapping)],
                        kw_vararg: &Option<String>) 
                        -> ExecutionResult {

    let mut dependencies = vec!();
    let mut changes = vec!();

    if let Some(ref target) = *kw_vararg {
        let str_type = "str".to_owned();
        let str_ptr = vm.object_of_type(&str_type);

        let dict_type = "dict".to_owned();
        let dict_ptr = vm.object_of_type(&dict_type);

        let set_type = "set".to_owned();
        let keys_ptr = vm.object_of_type(&set_type);
        let values_ptr = vm.object_of_type(&set_type);

        let mut key_chunks = Vec::new();
        let mut value_chunks = Vec::new();

        for arg in gkw.iter() {
            let &(_, ref mapping) = arg;
            // define the keys
            let mut chunk = CollectionChunk::empty();
            let kind = vm.get_object(&str_ptr).get_extension().first().unwrap();
            let repr = Representant::new(str_ptr, *kind, Some(1), Some(1));
            chunk.add_representant(Path::empty(), repr);  
            key_chunks.push(chunk); 

            let mut chunk = CollectionChunk::empty();
            for &(ref path, ref address) in mapping {
                let kind = vm.get_object(address).get_extension().first().unwrap();
                let repr = Representant::new(*address, *kind, Some(1), Some(1));
                chunk.add_representant(path.clone(), repr);    
            }

            value_chunks.push(chunk);
        }

        {
            let mut obj = vm.get_object_mut(&keys_ptr);
            obj.define_elements(key_chunks, Path::empty());
        }

        {
            let mut obj = vm.get_object_mut(&values_ptr);
            obj.define_elements(value_chunks, Path::empty());
        }

        {
            let mut obj = vm.get_object_mut(&dict_ptr);
            let keys_mapping = Mapping::simple(Path::empty(), keys_ptr);
            let values_mapping = Mapping::simple(Path::empty(), values_ptr);

            obj.assign_attribute("___keys".to_owned(), Path::empty(), keys_mapping);
            obj.assign_attribute("___values".to_owned(), Path::empty(), values_mapping);
        }

        let mapping = Mapping::simple(vm.current_path().clone(), dict_ptr);
        let mut aresult = vm.assign_direct(executors, target.clone(), mapping); 
        dependencies.append(&mut aresult.dependencies);
        changes.append(&mut aresult.changes);
    }

    let mut aresult = assign_defaults(vm, executors, arg); 
    dependencies.append(&mut aresult.dependencies);
    changes.append(&mut aresult.changes);

    ExecutionResult {
        flow: FlowControl::Continue,
        dependencies: dependencies,
        changes: changes,
        result: Mapping::new(),
    }
}                 


fn assign_defaults(vm: &mut VirtualMachine,
                        executors: &Executors,
                        arg: &[(String, Mapping)]) 
                        -> ExecutionResult {

    let mut dependencies = vec!();
    let mut changes = vec!();

    for &(ref name, ref mapping) in arg.iter() {
        let mut aresult = vm.assign_direct(executors, name.clone(), mapping.clone());
        dependencies.append(&mut aresult.dependencies);
        changes.append(&mut aresult.changes);
    }

    ExecutionResult {
        flow: FlowControl::Continue,
        dependencies: dependencies,
        changes: changes,
        result: Mapping::new(),
    }
}