use super::Pointer;
use super::Assumption;

use std::collections::HashMap;
use std::collections::hash_map::Iter;

#[derive(Debug, Clone)]
pub struct Mapping {
    possibilities: HashMap<Assumption, Pointer>,
}

impl Mapping {
    pub fn new() -> Mapping {
        Mapping { possibilities: HashMap::new() }
    }

    pub fn simple(ass: Assumption, address: Pointer) -> Mapping {
        let mut map = HashMap::new();
        map.insert(ass, address);
        Mapping { possibilities: map }
    }

    pub fn add_mapping(&mut self, ass: Assumption, address: Pointer) {
        self.possibilities.insert(ass, address);
    }

    pub fn iter(&self) -> Iter<Assumption, Pointer> {
        return self.possibilities.iter();
    }
}

#[derive(Debug, Clone)]
pub struct OptionalMapping {
    possibilities: HashMap<Assumption, Option<Pointer>>,
}

impl OptionalMapping {
    pub fn new() -> OptionalMapping {
        OptionalMapping { possibilities: HashMap::new() }
    }

    pub fn add_mapping(&mut self, ass: Assumption, address: Option<Pointer>) {
        self.possibilities.insert(ass, address);
    }

    pub fn iter(&self) -> Iter<Assumption, Option<Pointer>> {
        return self.possibilities.iter();
    }
}