use super::Pointer;
use super::{Path, PathNode};
use super::PathID;

use std::vec::IntoIter;
use std::slice::Iter;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Mapping {
    possibilities: Vec<(Path, Pointer)>,
}

impl Mapping {
    pub fn new() -> Mapping {
        Mapping { possibilities: Vec::new() }
    }

    pub fn simple(path: Path, address: Pointer) -> Mapping {
        let mut map = Vec::new();
        map.push((path, address));
        Mapping { possibilities: map }
    }

    pub fn add_mapping(&mut self, path: Path, address: Pointer) {
        self.possibilities.push((path, address));
    }

    pub fn iter(&self) -> Iter<(Path, Pointer)> {
        return self.possibilities.iter();
    }

    pub fn into_iter(self) -> IntoIter<(Path, Pointer)> {
        return self.possibilities.into_iter();
    }

    pub fn augment(self, node: PathNode) -> Mapping {
        let mut new_possibilities = Vec::new();
        for (mut path, address) in self.possibilities.into_iter() {
            path.add_node(node.clone());
            new_possibilities.push((path.clone(), address));
        }
        return Mapping { possibilities: new_possibilities };
    }

    pub fn len(&self) -> usize {
        return self.possibilities.len();
    }

    pub fn prune(&self, cutoff: &PathID) -> Mapping {
        let mut new = Mapping::new();
        for &(ref path, ref address) in self.iter() {
            new.add_mapping(path.prune(cutoff), *address);
        }
        return new;
    }
}

#[derive(Debug, Clone)]
pub struct OptionalMapping {
    possibilities: Vec<(Path, Option<Pointer>)>,
}

impl OptionalMapping {
    pub fn new() -> OptionalMapping {
        OptionalMapping { possibilities: Vec::new() }
    }

    pub fn add_mapping(&mut self, path: Path, address: Option<Pointer>) {
        self.possibilities.push((path, address));
    }

    pub fn iter(&self) -> Iter<(Path, Option<Pointer>)> {
        return self.possibilities.iter();
    }

    pub fn into_iter(self) -> IntoIter<(Path, Option<Pointer>)> {
        return self.possibilities.into_iter();
    }

    pub fn len(&self) -> usize {
        return self.possibilities.len();
    }

    pub fn augment(self, node: PathNode) -> OptionalMapping {
        let mut new_possibilities = Vec::new();
        for (mut path, opt_address) in self.possibilities.into_iter() {
            path.add_node(node.clone());
            new_possibilities.push((path.clone(), opt_address));
        }
        return OptionalMapping { possibilities: new_possibilities };
    }
}
