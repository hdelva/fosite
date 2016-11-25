use std::collections::HashMap;

use super::Pointer;
use super::Object;

use super::FunctionDefinition;
use super::BuiltinFunction;

pub struct Memory {
    count: i16,

    pointer_chain: HashMap<Pointer, Pointer>,

    objects: HashMap<Pointer, Object>,

    function_definitions: HashMap<Pointer, FunctionDefinition>,
    builtin_functions: HashMap<Pointer, BuiltinFunction>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            count: 0,
            pointer_chain: HashMap::new(),
            objects: HashMap::new(),
            function_definitions: HashMap::new(),
            builtin_functions: HashMap::new(),
        }
    }

    /// add new things to the memory
    pub fn new_object(&mut self) -> Pointer {
        let index = self.count;
        self.count += 1;

        let object = Object::new();
        self.objects.insert(index, object);

        return index
    }

    pub fn follow_pointer_chain<'a>(&'a self, address: &'a Pointer) -> &'a Pointer {
        let mut current = address;

        loop {
            match self.pointer_chain.get(current) {
                Some(next) => {
                    if next > current {
                        panic!("Corrupted UF structure")
                    }

                    current = next;
                },
                None => return current
            }
        }
    }

    pub fn chain_pointers(&mut self, first: &Pointer, second: &Pointer) {
        let smallest;
        let largest;

        if first < second {
            smallest = first;
            largest = second;
        } else {
            smallest = second;
            largest = first;
        }

        self.pointer_chain.insert(largest.clone(), smallest.clone());
    }

    pub fn free(&mut self, address: &Pointer) {
        self.objects.remove(address);
    }


    pub fn define_custom_function(&mut self, defintion: FunctionDefinition) {
        let index = self.count;
        self.count += 1;

        self.function_definitions.insert(index, defintion);
    }

    pub fn define_builtin_function(&mut self, definition: BuiltinFunction) {
        let index = self.count;
        self.count += 1;

        self.builtin_functions.insert(index, definition);
    }

    /// get things from the memory
    pub fn get_object(&self, address: &Pointer) -> &Object {
        match self.objects.get(address) {
            Some(value) => return value,
            None => panic!("Invalid Pointer Value")
        }
    }

    /// get things from the memory
    pub fn get_object_mut(&mut self, address: &Pointer) -> &mut Object {
        match self.objects.get_mut(address) {
            Some(mut value) => return value,
            None => panic!("Invalid Pointer Value")
        }
    }


    pub fn get_function_definition(&self, address: &Pointer) -> &FunctionDefinition {
        match self.function_definitions.get(address) {
            Some(def) => {
                return def;
            },
            None => panic!("Invalid Pointer Value")
        }
    }

    pub fn get_builtin_function(&self, address: &Pointer) -> &BuiltinFunction {
        match self.builtin_functions.get(address) {
            Some(def) => {
                return def;
            },
            None => panic!("Invalid Pointer Value")
        }
    }
}