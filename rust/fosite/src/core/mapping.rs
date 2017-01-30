use super::Pointer;
use super::{Path, PathNode};

use std::collections::HashMap;
use std::collections::hash_map::{Iter, IntoIter};

#[derive(Debug, Clone)]
pub struct Mapping {
    possibilities: HashMap<Path, Pointer>,
}

impl Mapping {
    pub fn new() -> Mapping {
        Mapping { possibilities: HashMap::new() }
    }

    pub fn simple(path: Path, address: Pointer) -> Mapping {
        let mut map = HashMap::new();
        map.insert(path, address);
        Mapping { possibilities: map }
    }

    pub fn add_mapping(&mut self, path: Path, address: Pointer) {
        self.possibilities.insert(path, address);
    }

    pub fn iter(&self) -> Iter<Path, Pointer> {
        return self.possibilities.iter();
    }

    pub fn augment(self, node: PathNode) -> Mapping {
        let mut new_possibilities = HashMap::new();
        for (mut path, address) in self.possibilities.into_iter() {
            path.add_node(node.clone());
            new_possibilities.insert(path.clone(), address);
        }
        return Mapping {possibilities: new_possibilities};
    }
}

#[derive(Debug, Clone)]
pub struct OptionalMapping {
    possibilities: HashMap<Path, Option<Pointer>>,
}

impl OptionalMapping {
    pub fn new() -> OptionalMapping {
        OptionalMapping { possibilities: HashMap::new() }
    }

    pub fn add_mapping(&mut self, path: Path, address: Option<Pointer>) {
        self.possibilities.insert(path, address);
    }

    pub fn iter(&self) -> Iter<Path, Option<Pointer>> {
        return self.possibilities.iter();
    }
    
    pub fn into_iter(self) -> IntoIter<Path, Option<Pointer>> {
    	return self.possibilities.into_iter();
    }
    
    pub fn len(&self) -> usize {
    	return self.possibilities.len();
    }
}
