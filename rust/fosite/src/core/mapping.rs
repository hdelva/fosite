use super::Pointer;
use super::{Path, PathNode};
use super::PathID;

use std::vec::IntoIter;
use std::slice::Iter;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
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

    pub fn _iter(&self) -> Iter<(Path, Pointer)> {
        self.possibilities.iter()
    }

    pub fn augment(self, node: PathNode) -> Mapping {
        let mut new_possibilities = Vec::new();
        for (mut path, address) in self.possibilities {
            path.add_node(node.clone());
            new_possibilities.push((path.clone(), address));
        }

        Mapping { possibilities: new_possibilities }
    }

    pub fn len(&self) -> usize {
        self.possibilities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[allow(ptr_arg)]
    pub fn prune(&self, cutoff: &PathID) -> Mapping {
        let mut new = Mapping::new();
        for &(ref path, ref address) in self {
            new.add_mapping(path.prune(cutoff), *address);
        }

        new
    }
}

impl<'a> IntoIterator for &'a Mapping {
    type Item = &'a (Path, Pointer);
    type IntoIter = MappingIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MappingIterator {mapping: self, current: 0}
    }
}

pub struct MappingIterator<'a> {
    mapping: &'a Mapping,
    current: usize,
}

impl<'a> Iterator for MappingIterator<'a> {
    type Item = &'a (Path, Pointer);

    fn next(&mut self) -> Option<&'a (Path, Pointer)> {
        let result = self.mapping.possibilities.get(self.current);
        self.current += 1;
        result
    }
}

impl IntoIterator for Mapping {
    type Item = (Path, Pointer);
    type IntoIter = IntoIter<(Path, Pointer)>;

    fn into_iter(self) -> Self::IntoIter {
        self.possibilities.into_iter()
    }
}

#[derive(Debug, Clone, Default)]
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

    pub fn _iter(&self) -> Iter<(Path, Option<Pointer>)> {
        self.possibilities.iter()
    }

    pub fn len(&self) -> usize {
        self.possibilities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn augment(self, node: PathNode) -> OptionalMapping {
        let mut new_possibilities = Vec::new();
        for (mut path, opt_address) in self.possibilities {
            path.add_node(node.clone());
            new_possibilities.push((path.clone(), opt_address));
        }

        OptionalMapping { possibilities: new_possibilities }
    }
}

impl<'a> IntoIterator for &'a OptionalMapping {
    type Item = &'a (Path, Option<Pointer>);
    type IntoIter = OptionalMappingIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        OptionalMappingIterator {mapping: self, current: 0}
    }
}

pub struct OptionalMappingIterator<'a> {
    mapping: &'a OptionalMapping,
    current: usize,
}

impl<'a> Iterator for OptionalMappingIterator<'a> {
    type Item = &'a (Path, Option<Pointer>);

    fn next(&mut self) -> Option<&'a (Path, Option<Pointer>)> {
        let result = self.mapping.possibilities.get(self.current);
        self.current += 1;
        result
    }
}

impl IntoIterator for OptionalMapping {
    type Item = (Path, Option<Pointer>);
    type IntoIter = IntoIter<(Path, Option<Pointer>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.possibilities.into_iter()
    }
}