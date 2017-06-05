use core::*;

use super::check_arg;

pub fn new_list_module() -> Module {
    let mut list = Module::new();
    define_append(&mut list);
    define_reverse(&mut list);
    list
}

fn define_reverse(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = *vm.knowledge().get_type(&"list".to_owned()).unwrap();

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let total_changes = Vec::new();
            let total_dependencies = Vec::new();

            let Environment { vm, .. } = env;

            if !args.is_empty() {
                check_arg(vm, &args[0], "first", vec!("collection"));
            }

            let type_name = "list".to_owned();
            let list_ptr = vm.object_of_type(&type_name);

            let mut content = Vec::new();

            for mapping in &args {
                for &(ref path, ref address) in mapping {
                    let old_object = vm.get_object(address);
                    let elements = old_object.get_elements().get_content();
                    for collection_mapping in elements.iter().rev() {
                        let mut new_path = collection_mapping.path.clone();
                        new_path.merge_into(path.clone());
                        content.push((new_path, collection_mapping.branch.clone()));
                    }
                }
            }

            let mut collection = Collection::new();
            collection.set_content(content);

            {
                let mut list_object = vm.get_object_mut(&list_ptr);
                list_object.set_elements(collection);
            }

            let mapping = Mapping::simple(Path::empty(), list_ptr);

            let path = vm.current_path().clone();
            vm.add_result(path, mapping);

            ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: total_dependencies,
                changes: total_changes,
                result: Mapping::new(),
            }
        };

        vm.set_callable(pointer, inner);

        pointer
    };
    
    module.add_part("reverse".to_owned(), Box::new(outer));
}

fn define_append(module: &mut Module) {
    let outer = |vm: &mut VirtualMachine| {
        let pointer = vm.object_of_type(&"method".to_owned());

        let inner = | env: Environment, args: Vec<Mapping>, _: Vec<(String, Mapping)> | {
            let Environment { vm, .. } = env;

            if args.len() == 2 {
                check_arg(vm, &args[0], "first", vec!("list"));
                check_arg(vm, &args[1], "second", vec!("object"));

                let current_path = vm.current_path().clone();
                let this = &args[0];
                let chunk = make_chunk(vm, &args[1]);

                for &(_, ref target_address) in this {
                    // remember the type of the collection before the addition
                    let original_type;
                    {
                        let parent_object = vm.get_object(target_address);
                        let kb = vm.knowledge();
                        original_type = parent_object.get_type_name(kb);
                    }

                    // add the new element
                    {
                        let mut parent_object = vm.get_object_mut(target_address);

                        // add the new element
                        parent_object.append_element(chunk.clone(), current_path.clone());
                    }

                    // get the new type of the
                    let new_type;
                    {
                        let parent_object = vm.get_object(target_address);
                        let kb = vm.knowledge();
                        new_type = parent_object.get_type_name(kb);
                    }

                    // check whether or not an element of a new type had been added
                    if !new_type.contains(&original_type) {
                        let content = HeteroCollection::new("object".to_owned(), original_type, new_type);
                        let message = Message::Output {
                            source: vm.current_node().clone(),
                            content: Box::new(content),
                        };
                        CHANNEL.publish(message);
                    }
                }
            }
            
            let mapping = Mapping::simple(Path::empty(), vm.knowledge().constant("None"));

            let path = vm.current_path().clone();
            vm.add_result(path, mapping);

            ExecutionResult {
                flow: FlowControl::Continue,
                dependencies: vec!(),
                changes: vec!(),
                result: Mapping::new(),
            }
        };

        vm.set_callable(pointer, inner);

        pointer
    };
    
    module.add_part("append".to_owned(), Box::new(outer));
}


fn make_chunk(vm: &VirtualMachine, mapping: &Mapping) -> CollectionChunk {
    let mut chunk = CollectionChunk::empty();

    let mut max = Some(1);
    for node in vm.current_path()._iter().rev() {
        match *node {
            PathNode::Loop(_) => {
                max = None;
                break;
            },
            PathNode::Frame(_, _, _, _) => {
                break;
            },
            _ => ()
        }
    }

    for &(ref path, ref pointer) in mapping {
        let value_obj = vm.get_object(pointer);
        let kind = value_obj.get_extension().first().unwrap();

        chunk.add_representant(path.clone(), Representant::new(*pointer, *kind, Some(0), max));
    }

    chunk
}