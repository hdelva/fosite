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

        let mut closure = Scope::new();
        let mut identifiers = BTreeSet::new();
        capture(body, &mut identifiers);

        for identifier in identifiers.iter() {
            let mut aresult = vm.execute(executors, identifier);
            dependencies.append(&mut aresult.dependencies);

            let name = identifier.to_string();
            closure.set_mapping(name, Path::empty(), aresult.result);
        }

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

        let index = ARGS.lock().unwrap().len();

        ARGS.lock().unwrap().push(rpos_evaluated);
        KW_ARGS.lock().unwrap().push(rkw_evaluated);
        VARARG.lock().unwrap().push(vararg.clone());
        KW_VARARG.lock().unwrap().push(kw_vararg.clone());
        BODY.lock().unwrap().push(body.clone());

        let inner = move | env: Environment, args: Vec<Mapping>, kw_args: Vec<(String, Mapping)> | {
            let Environment { vm, executors } = env;

            let new_node = vm.current_path().iter().last().unwrap().clone(); // should be the function call node

            let mut aug_args = Vec::new();
            let mut aug_kwargs = Vec::new();

            for &(ref n, ref a) in ARGS.lock().unwrap()[index].iter() {
                aug_args.push( (n.clone(), a.clone().augment(new_node.clone())) );
            }

            for &(ref n, ref a) in KW_ARGS.lock().unwrap()[index].iter() {
                aug_kwargs.push( (n.clone(), a.clone().augment(new_node.clone())) );
            }

            assign_positional(vm, executors,
                        &aug_args, &aug_kwargs, 
                        &args, &kw_args,
                        &VARARG.lock().unwrap()[index], &KW_VARARG.lock().unwrap()[index]);

            let body_result = vm.execute(executors, &BODY.lock().unwrap()[index]);

            
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
        vm.set_closure(pointer.clone(), closure);

        let mut aresult = vm.assign_direct(executors, name.clone(), Mapping::simple(Path::empty(), pointer));
        changes.append(&mut aresult.changes);
        dependencies.append(&mut aresult.dependencies);

        return ExecutionResult {
            flow: FlowControl::Continue,
            changes: changes,
            dependencies: dependencies,
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

    // todo, use the default value before moving to varargs
    if rpos.len() > 0 && gpos.len() > 0 {
        let &(ref name, _) = &rpos[0];
        let mapping = gpos[0].clone();
        let mut aresult = vm.assign_direct(executors, name.clone(), mapping);
        dependencies.append(&mut aresult.dependencies);
        changes.append(&mut aresult.changes);
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

    if let &Some(ref name) = vararg {
        let type_name = "list".to_owned();
        let obj_ptr = vm.object_of_type(&type_name);        

        let mut chunks = Vec::new();
        for arg in gpos.iter() {
            let mut chunk = CollectionChunk::empty();

            for (path, address) in arg.iter(){
                let kind = vm.get_object(address).get_extension().first().unwrap();
                let repr = Representant::new(address.clone(), kind.clone(), Some(1), Some(1));
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
        let &(ref name, _) = &rkw[0];
        let mapping = gpos[0].clone();
        let mut aresult = vm.assign_direct(executors, name.clone(), mapping);
        dependencies.append(&mut aresult.dependencies);
        changes.append(&mut aresult.changes);

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

        let mut aresult = vm.assign_direct(executors, name.clone(), mapping);
        dependencies.append(&mut aresult.dependencies);
        changes.append(&mut aresult.changes);

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

    if let &Some(ref target) = kw_vararg {
        let str_type = "string".to_owned();
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
            let repr = Representant::new(str_ptr, kind.clone(), Some(1), Some(1));
            chunk.add_representant(Path::empty(), repr);  
            key_chunks.push(chunk); 

            let mut chunk = CollectionChunk::empty();
            for (path, address) in mapping.iter(){
                let kind = vm.get_object(address).get_extension().first().unwrap();
                let repr = Representant::new(address.clone(), kind.clone(), Some(1), Some(1));
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
            let keys_mapping = Mapping::simple(Path::empty(), keys_ptr.clone());
            let values_mapping = Mapping::simple(Path::empty(), values_ptr.clone());

            obj.assign_attribute("___keys".to_owned(), Path::empty(), keys_mapping);
            obj.assign_attribute("___values".to_owned(), Path::empty(), values_mapping);
        }

        let mapping = Mapping::simple(vm.current_path().clone(), dict_ptr);
        let mut aresult = vm.assign_direct(executors, target.clone(), mapping); 
        dependencies.append(&mut aresult.dependencies);
        changes.append(&mut aresult.changes);
    }

    return ExecutionResult {
        flow: FlowControl::Continue,
        dependencies: dependencies,
        changes: changes,
        result: Mapping::new(),
    }
}           

fn capture(node: &GastNode, acc: &mut BTreeSet<GastNode>) {
    match &node.kind {
        &NodeType::Boolean { .. } => (),
        &NodeType::String { .. } => (),
        &NodeType::Int { .. } => (),
        &NodeType::Float { .. } => (),
        &NodeType::Nil {} => (),
        &NodeType::BinOp { ref left, ref right, .. } => {
            capture(left, acc);
            capture(right, acc);
        }
        &NodeType::BoolOp { ref left, ref right, .. } => {
            capture(left, acc);
            capture(right, acc);
        }
        &NodeType::If { ref test, ref body, ref or_else } => {
            capture(test, acc);
            capture(body, acc);
            capture(or_else, acc);
        }
        &NodeType::Block { ref content } => {
            for n in content.iter() {
                capture(n, acc);
            }
        }
        &NodeType::Identifier { .. } => {
            acc.insert(node.clone());
        }
        &NodeType::Attribute { ref parent, .. } => {
            capture(parent, acc);
        }
        &NodeType::Declaration { .. } => (),
        &NodeType::Assignment { ref value, .. } => {
            capture(value, acc);
        }
        &NodeType::While { ref test, ref body } => {
            capture(test, acc);
            capture(body, acc);
        }
        &NodeType::Break {  } => (),
        &NodeType::Continue {  } => (),
        &NodeType::List { ref content } => {
            for n in content.iter() {
                capture(n, acc);
            }
        }
        &NodeType::Set { ref content } => {
            for n in content.iter() {
                capture(n, acc);
            }
        }
        &NodeType::Sequence {ref content } => {
            for n in content.iter() {
                capture(n, acc);
            }
        }
        &NodeType::Index {ref target, ref index} => {
            capture(target, acc);
            capture(index, acc);
        }
        &NodeType::Dict {ref content} => {
            for n in content.iter() {
                capture(n, acc);
            }
        }
        &NodeType::Generator {ref source, ..} => {
            capture(source, acc);
        }
        &NodeType::Filter {ref source, ref condition} => {
            capture(source, acc);
            capture(condition, acc);
        }
        &NodeType::Map {ref source, ..} => {
            capture(source, acc);
        }
        &NodeType::AndThen {ref first, ref second} => {
            capture(first, acc);
            capture(second, acc);
        }
        &NodeType::ForEach {ref before, ref body} => {
            capture(before, acc);
            capture(body, acc);
        }
        &NodeType::Call {ref target, ref args, ref kwargs} => {
            capture(target, acc);
            
            for n in args.iter() {
                capture(n, acc);
            }

            for n in kwargs.iter() {
                capture(n, acc);
            }
        }
        &NodeType::Import {..} => (),
        &NodeType::Negate {ref value} => {
            capture(value, acc);
        }
        &NodeType::UnOp {ref value, ..} => {
            capture(value, acc);
        }
        &NodeType::Slice {ref target, ref lower, ref upper} => {
            capture(target, acc);
            capture(lower, acc);
            capture(upper, acc);
        }
        &NodeType::FunctionDef {..} => (),
        &NodeType::Return {ref value} => {
            capture(value, acc);
        }
        &NodeType::Pair {ref first, ref second} => {
            capture(first, acc);
            capture(second, acc);
        }
        &NodeType::Argument { ref value, .. } => {
            capture(value, acc);
        }
    }
}      
