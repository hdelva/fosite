use super::Pointer;
use super::Assumption;

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

#[derive(Debug, Clone)]
pub struct Mapping {
    pub assumption: Assumption,
    pub address: Pointer,
}

impl Mapping {
    pub fn new(assumption: Assumption, address: Pointer) -> Mapping {
        Mapping {
            assumption: assumption,
            address: address,
        }
    }
}

pub struct Scope {
    // depending on the execution path, a variable name can refer to multiple things
    identifiers: HashMap<String, Vec<Mapping>>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope { identifiers: HashMap::new() }
    }

    pub fn invalidate_mappings(&mut self, name: &String) {
        self.identifiers.remove(name);
    }

    pub fn add_mapping(&mut self, name: &String, mapping: Mapping) {
        match self.identifiers.entry(name.clone()) {
            Vacant(entry) => {
                let list = entry.insert(Vec::new());
                list.push(mapping);
            }
            Occupied(mut entry) => {
                let list = entry.get_mut();
                list.push(mapping);
            }
        }
    }

    pub fn resolve_identifier(&self, name: &String) -> Option<&Vec<Mapping>> {
        match self.identifiers.get(name) {
            Some(mappings) => return Some(mappings),
            None => None,
        }
    }

    // legacy code
    //
    // takes ownership of self, dies in the process
    // pub fn merge_into(self, target: &mut Scope) {
    // for (name, mut list) in self.identifiers {
    // match target.identifiers.entry(name) {
    // Vacant(entry) => {
    // let own_list = entry.insert(Vec::new());
    // own_list.append(&mut list);
    // },
    // Occupied(mut entry) => {
    // let result = Vec::new();
    //
    // let own_list go out of scope before inserting
    // {
    // take ownership of the current content, effectively removing it
    // let own_list = entry.get();
    //
    // improved zip, more efficient as well
    // let longest_list;
    // let shortest_list;
    //
    // if list.len() > own_list.len() {
    // longest_list = &list;
    // shortest_list = own_list;
    // } else {
    // longest_list = own_list;
    // shortest_list = &list;
    // }
    //
    // let ref cut = longest_list[..shortest_list.len()];
    //
    // let mut result = Vec::new();
    //
    // merge whatever they have in common
    // for i in 0..cut.len() {
    // let ref x = longest_list[i];
    // let ref y = shortest_list[i];
    //
    // let merged: HashSet<Pointer> = x.union(&y).cloned().collect();
    // result.push(merged);
    // }
    //
    // add the rest
    // for i in cut.len()..longest_list.len() {
    // let x: HashSet<Pointer> = longest_list[i].clone();
    // result.push(x);
    // }
    // }
    //
    // put the result back in the hashmap
    // entry.insert(result);
    // }
    // }
    // }
    // }
}
