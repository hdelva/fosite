use super::Pointer;
use super::{Path, PathNode};
use super::PathID;

use std::collections::BTreeMap;
use std::collections::btree_map::{Iter, IntoIter};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Mapping {
    possibilities: BTreeMap<Path, Pointer>,
}

impl Mapping {
    pub fn new() -> Mapping {
        Mapping { possibilities: BTreeMap::new() }
    }

    pub fn simple(path: Path, address: Pointer) -> Mapping {
        let mut map = BTreeMap::new();
        map.insert(path, address);
        Mapping { possibilities: map }
    }

    pub fn add_mapping(&mut self, path: Path, address: Pointer) {
        self.possibilities.insert(path, address);
    }

    pub fn iter(&self) -> Iter<Path, Pointer> {
        return self.possibilities.iter();
    }

    pub fn into_iter(self) -> IntoIter<Path, Pointer> {
        return self.possibilities.into_iter();
    }

    pub fn augment(self, node: PathNode) -> Mapping {
        let mut new_possibilities = BTreeMap::new();
        for (mut path, address) in self.possibilities.into_iter() {
            path.add_node(node.clone());
            new_possibilities.insert(path.clone(), address);
        }
        return Mapping { possibilities: new_possibilities };
    }

    pub fn len(&self) -> usize {
        return self.possibilities.len();
    }

    pub fn prune(&self, cutoff: &PathID) -> Mapping {
        let mut new = Mapping::new();
        for (path, address) in self.iter() {
            new.add_mapping(path.prune(cutoff), *address);
        }
        return new;
    }
}

#[derive(Debug, Clone)]
pub struct OptionalMapping {
    possibilities: BTreeMap<Path, Option<Pointer>>,
}

impl OptionalMapping {
    pub fn new() -> OptionalMapping {
        OptionalMapping { possibilities: BTreeMap::new() }
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

    pub fn augment(self, node: PathNode) -> OptionalMapping {
        let mut new_possibilities = BTreeMap::new();
        for (mut path, opt_address) in self.possibilities.into_iter() {
            path.add_node(node.clone());
            new_possibilities.insert(path.clone(), opt_address);
        }
        return OptionalMapping { possibilities: new_possibilities };
    }
}
