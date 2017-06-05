use std::collections::HashMap;

use super::Pointer;
use super::Object;

pub struct Memory {
    count: i16,

    pointer_chain: HashMap<Pointer, Pointer>,

    objects: HashMap<Pointer, Object>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            count: 0,
            pointer_chain: HashMap::new(),
            objects: HashMap::new(),
        }
    }

    /// add new things to the memory
    pub fn new_object(&mut self) -> Pointer {
        let index = self.count;
        self.count += 1;

        let object = Object::new();
        self.objects.insert(index, object);

        index
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
                }
                None => return current,
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

    /// get things from the memory
    pub fn get_object(&self, address: &Pointer) -> &Object {
        match self.objects.get(address) {
            Some(value) => value,
            None => panic!("Invalid Pointer Value"),
        }
    }

    // get things from the memory
    pub fn get_object_mut(&mut self, address: &Pointer) -> &mut Object {
        match self.objects.get_mut(address) {
            Some(mut value) => value,
            None => panic!("Invalid Pointer Value"),
        }
    }
}
