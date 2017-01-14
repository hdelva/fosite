use super::Pointer;
use bidir_map::BidirMap;
use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};


pub struct KnowledgeBase {
    types: BidirMap<String, Pointer>,
    callable_types: BidirMap<String, Pointer>,
    iterable_types: BidirMap<String, Pointer>,
    indexable_types: BidirMap<String, Pointer>,
    constants: HashMap<String, Pointer>,
    arithmetic_types: HashMap<String, HashSet<String>>,
}

impl KnowledgeBase {
    pub fn new() -> KnowledgeBase {
        KnowledgeBase {
            types: BidirMap::new(),
            constants: HashMap::new(),
            callable_types: BidirMap::new(),
            iterable_types: BidirMap::new(),
            indexable_types: BidirMap::new(),
            arithmetic_types: HashMap::new(),
        }
    }

    pub fn add_arithmetic_type(&mut self, name: &str, op: &str) {
        match self.arithmetic_types.entry(name.to_owned()) {
            Occupied(mut ops) => {
                let mut set = ops.get_mut();
                set.insert(op.to_owned());
            },
            Vacant(entry) => {
                let mut set = HashSet::new();
                set.insert(op.to_owned());
                entry.insert(set);
            }
        }
    }

    pub fn add_constant(&mut self, name: &String, address: &Pointer) {
        self.constants.insert(name.clone(), address.clone());
    }

    pub fn constant(&self, name: &str) -> Pointer {
        // need this to assign the None constant :|
        if name == "None" {
            return 2
        }

        return self.constants.get(name).unwrap().clone();
    }

    pub fn operation_supported(&self, type_name: &String, operation: &String) -> bool {
        match self.arithmetic_types.get(type_name) {
            Some(ops) => return ops.contains(operation),
            _ => false,
        }
    }

    pub fn add_callable_type(&mut self, name: String, address: Pointer) {
        self.callable_types.insert(name.clone(), address.clone());
        self.types.insert(name, address);
    }

    pub fn add_iterable_type(&mut self, name: String, address: Pointer) {
        self.iterable_types.insert(name.clone(), address.clone());
        self.types.insert(name, address);
    }

    pub fn add_indexable_type(&mut self, name: String, address: Pointer) {
        self.indexable_types.insert(name.clone(), address.clone());
        self.types.insert(name, address);
    }

    pub fn add_type(&mut self, name: String, address: Pointer) {
        self.types.insert(name, address);
    }

    pub fn get_type(&self, name: &String) -> Option<&Pointer> {
        return self.types.get_by_first(name);
    }

    pub fn get_type_name(&self, pointer: &Pointer) -> &String {
        return self.types.get_by_second(pointer).expect("No type at this address");
    }
}
