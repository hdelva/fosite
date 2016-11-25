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

    types_with_attribute: HashMap<String, HashSet<Pointer>>,
}

impl KnowledgeBase {
    pub fn new() -> KnowledgeBase {
        KnowledgeBase {
            types: BidirMap::new(),
            callable_types: BidirMap::new(),
            iterable_types: BidirMap::new(),
            indexable_types: BidirMap::new(),
            types_with_attribute: HashMap::new(),
        }
    }

    pub fn add_type_with_attribute(&mut self, attr: String, address: Pointer) {
        if !self.types.contains_second_key(&address){
            panic!("Referring to non-existing type")
        }

        match self.types_with_attribute.entry(attr.clone()){
            Occupied(mut entry) => {
                let mut set = entry.get_mut();
                set.insert(address.clone());
            },
            Vacant(entry) => {
                let mut set = HashSet::new();
                set.insert(address.clone());
                entry.insert(set);
            },
        };
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

    pub fn get_type(&self, name: &String) -> Option<&Pointer> {
        return self.types.get_by_first(name)
    }


    pub fn get_types_with_attribute(&self, name: &String) -> Option<&HashSet<Pointer>> {
        return self.types_with_attribute.get(name)
    }
}