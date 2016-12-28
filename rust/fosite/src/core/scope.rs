use super::Assumption;
use super::{Mapping, OptionalMapping};

use std::collections::HashSet;
use std::collections::HashMap;

struct Frame {
    content: HashMap<String, OptionalMapping>,
    assumption: Assumption,
    parent: Option<usize>,
}

impl Frame {
    fn new(assumption: Assumption, parent: Option<usize>) -> Frame {

        Frame {
            assumption: assumption,
            parent: parent,
            content: HashMap::new(),
        }
    }

    fn resolve_identifier(&self, name: &String) -> Option<&OptionalMapping> {
        return self.content.get(name);
    }

    fn set_optional_mapping(&mut self, name: String, mapping: OptionalMapping) {
        self.content.insert(name, mapping);
    }

    fn set_mapping(&mut self, name: String, mapping: Mapping) {
        let mut new_mapping = OptionalMapping::new();

        for (ass, address) in mapping.iter() {
            new_mapping.add_mapping(ass.clone(), Some(address.clone()));
        }

        self.set_optional_mapping(name, new_mapping);
    }

    fn parent_index(&self) -> Option<usize> {
        return self.parent;
    }

    fn get_assumption(&self) -> &Assumption {
        return &self.assumption;
    }

    fn get_content(&self) -> &HashMap<String, OptionalMapping> {
        return &self.content;
    }
}

pub struct Scope {
    frames: Vec<Frame>,
    path: Vec<bool>,
    default: OptionalMapping,
}

impl Scope {
    pub fn new() -> Scope {
        let mut default = OptionalMapping::new();
        default.add_mapping(Assumption::empty(), None);

        Scope {
            frames: vec![Frame::new(Assumption::empty(), None)],
            default: default,
            path: vec![false],
        }
    }

    pub fn resolve_identifier(&self, name: &String) -> Mapping {
        let mut mapping = Mapping::new();

        for (assumption, opt_address) in self.resolve_optional_identifier(name).iter() {
            if let &Some(address) = opt_address {
                mapping.add_mapping(assumption.clone(), address);
            } else {
                panic!("no valid mapping under current assumption")
            }
        }

        return mapping;
    }

    pub fn resolve_optional_identifier(&self, name: &String) -> &OptionalMapping {
        let mut current_index = self.frames.len() - 1;

        if *self.path.last().unwrap() {
            current_index -= 1
        }


        loop {
            if let Some(result) = self.frames[current_index].resolve_identifier(name) {
                return result;
            }

            if let Some(parent) = self.frames[current_index].parent_index() {
                current_index = parent;
            } else {
                return &self.default;
            }

        }
    }

    pub fn set_mapping(&mut self, name: String, assumption: Assumption, mapping: Mapping) {
        let mut current_index = 0 as usize;

        for &(source, positive) in assumption.iter() {
            let old_index = current_index;

            if positive {
                current_index += 1
            } else {
                current_index += 2
            }

            if current_index >= self.frames.len() {
                let positive_assumption = Assumption::simple(source.clone(), positive.clone());
                self.frames.push(Frame::new(positive_assumption, Some(old_index.clone())));
                let negative_assumption = Assumption::simple(source.clone(), !positive);
                self.frames.push(Frame::new(negative_assumption, Some(old_index.clone())));
                self.path.push(true);
            }
        }

        self.frames[current_index].set_mapping(name, mapping)
    }

    pub fn change_branch(&mut self) {
        let current = self.path.pop().unwrap();
        self.path.push(!current);
    }

    pub fn merge_branches(&mut self) {
        let mut new_content = HashMap::new();

        let mut identifiers = HashSet::new();

        for name in self.frames[self.frames.len() - 2].get_content().keys() {
            identifiers.insert(name.clone());
        }

        for name in self.frames[self.frames.len() - 1].get_content().keys() {
            identifiers.insert(name.clone());
        }


        // first branch
        {
            let ref frame = self.frames[self.frames.len() - 1];
            let assumption = frame.get_assumption();

            let &(new_source, new_positive) = assumption.get().last().unwrap();

            for name in &identifiers {
                let old_mapping = self.resolve_optional_identifier(name);
                let mut new_mapping = OptionalMapping::new();

                for (old_assumption, address) in old_mapping.iter() {
                    let mut new_assumption = old_assumption.clone();
                    new_assumption.add(new_source.clone(), new_positive.clone());
                    new_mapping.add_mapping(new_assumption, address.clone());
                }

                new_content.insert(name.clone(), new_mapping);
            }
        }

        // hard coded swap
        {
            let current = self.path.pop().unwrap();
            self.path.push(!current);
        }

        // second branch
        {
            let ref frame = self.frames[self.frames.len() - 1];
            let assumption = frame.get_assumption();

            let &(new_source, new_positive) = assumption.get().last().unwrap();

            for name in &identifiers {
                let old_mapping = self.resolve_optional_identifier(name);
                let mut new_mapping = new_content.get_mut(name).unwrap();

                for (old_assumption, address) in old_mapping.iter() {
                    let mut new_assumption = old_assumption.clone();
                    new_assumption.add(new_source.clone(), new_positive.clone());
                    new_mapping.add_mapping(new_assumption, address.clone());
                }

            }
        }

        let _ = self.path.pop();
        let _ = self.frames.pop();
        let _ = self.frames.pop();

        let current_frame = self.frames.last_mut().unwrap();

        for (name, mapping) in new_content.into_iter() {
            current_frame.set_optional_mapping(name.clone(), mapping)
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
