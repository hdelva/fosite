use super::AnalysisItem;
use super::Mapping;
use super::Pointer;
use super::Path;
use super::PathID;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Watch {
    pub identifiers_before: HashMap<AnalysisItem, HashSet<Pointer>>,
    pub relevant_objects: HashSet<Pointer>,
    pub identifiers_changed: HashMap<AnalysisItem, Mapping>,
    pub objects_changed: HashMap<Pointer, Vec<Path>>,
    in_setup: bool,
    source: PathID,
}

impl Watch {
    pub fn new(source: PathID) -> Self {
        Watch {
            source: source,
            identifiers_before: HashMap::new(),
            relevant_objects: HashSet::new(),
            identifiers_changed: HashMap::new(),
            objects_changed: HashMap::new(),
            in_setup: true,
        }
    }

    pub fn toggle(&mut self) {
        self.in_setup = !self.in_setup;
    }

    pub fn store_identifier_dependency(&mut self, identifier: AnalysisItem, mapping: &Mapping) {
        if self.in_setup {
            for (_, address) in mapping.iter() {
                self.store_object_dependency(*address);
            }

            match self.identifiers_before.entry(identifier.clone()) {
                Entry::Occupied(mut m) => {
                    let mut acc = m.get_mut();
                    for (_, address) in mapping.iter() {
                        acc.insert(*address);
                    }
                },
                Entry::Vacant(m) => {
                    let mut acc = HashSet::new();
                    for (_, address) in mapping.iter() {
                        acc.insert(*address);
                    }
                    m.insert(acc);
                }
            }
        }
    }

    pub fn store_object_dependency(&mut self, address: Pointer) {
        if self.in_setup {
            self.relevant_objects.insert(address);
        }
    }

    pub fn store_identifier_change(&mut self, identifier: AnalysisItem, path: &Path, mapping: &Mapping) {
        if self.identifiers_before.contains_key(&identifier) {
            match self.identifiers_changed.entry(identifier) {
                Entry::Occupied(mut m) => {
                    let mut acc = m.get_mut();
                    for (other_path, address) in mapping.iter() {
                        let mut p1 = path.prune(&self.source);
                        let p2 = other_path.prune(&self.source);
                        p1.merge_into(p2);
                        acc.add_mapping(p1, *address);
                    }
                },
                Entry::Vacant(m) => {
                    let mut acc = Mapping::new();
                    for (other_path, address) in mapping.iter() {
                        let mut p1 = path.prune(&self.source);
                        let p2 = other_path.prune(&self.source);
                        p1.merge_into(p2);
                        acc.add_mapping(p1, *address);
                    }
                    m.insert(acc);
                }
            }
        }
    }

    pub fn store_object_change(&mut self, address: Pointer, path: &Path) {
        if self.relevant_objects.contains(&address) {
            match self.objects_changed.entry(address) {
                Entry::Occupied(mut m) => {
                    let mut acc = m.get_mut();
                    acc.push(path.prune(&self.source));
                },
                Entry::Vacant(m) => {
                    m.insert(vec!(path.prune(&self.source)));
                }
            }
        }
    }
}

