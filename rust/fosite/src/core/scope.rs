use super::{Path, PathNode};
use super::{Mapping, OptionalMapping};
use super::Pointer;

use std::collections::BTreeSet;
use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

#[derive(Debug)]
struct Frame {
    content: HashMap<String, OptionalMapping>,
    cause: PathNode,
    parent: Option<usize>,
}

impl Frame {
    fn new(cause: PathNode, parent: Option<usize>) -> Frame {
        Frame {
            cause: cause,
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

        for (path, address) in mapping.iter() {
            new_mapping.add_mapping(path.clone(), Some(address.clone()));
        }

        self.set_optional_mapping(name, new_mapping);
    }

    fn parent_index(&self) -> Option<usize> {
        return self.parent;
    }

    fn get_cause(&self) -> &PathNode {
        return &self.cause;
    }

    fn get_content(&self) -> &HashMap<String, OptionalMapping> {
        return &self.content;
    }
}

pub struct Scope {
    frames: Vec<Frame>,
    path: Vec<usize>,
    default: OptionalMapping,
    constants: BTreeSet<String>,
}

impl Scope {
    pub fn new() -> Scope {
        let mut default = OptionalMapping::new();
        default.add_mapping(Path::empty(), None);

        Scope {
            frames: vec![Frame::new(PathNode::Frame(0, None, BTreeSet::new()), None)],
            default: default,
            path: vec![0],
            constants: BTreeSet::new(),
        }
    }

    pub fn resolve_identifier(&self, name: &String) -> Mapping {
        let mut mapping = Mapping::new();

        for (path, opt_address) in self.resolve_optional_identifier(name).iter() {
            if let &Some(address) = opt_address {
                mapping.add_mapping(path.clone(), address);
            } else {
                panic!("no valid mapping under the current conditions")
            }
        }

        return mapping;
    }

    pub fn resolve_optional_identifier(&self, name: &String) -> &OptionalMapping {
        // last possible frame
        let mut current_index = self.frames.len() - 1;

        // path describes offset from the last frame
        current_index -= *self.path.last().unwrap();

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

    pub fn set_mapping(&mut self, name: String, path: Path, mapping: Mapping) {
        if self.constants.contains(&name) {
            // todo, throw error
            return
        }

        let mut count = 0 as usize;
        let mut current_index = 0 as usize;

        for node in path.iter() {
            let old_index = current_index;

            match node {
                &PathNode::Condition(_, b) | &PathNode::Loop(_, b) => {
                    count += 2;
                    if b {
                        current_index = count - 1;
                    } else {
                        current_index = count;
                    }
                },
                _ => {
                    count += 1;
                    current_index = count; 
                }
            }

            if current_index >= self.frames.len() {
                match node {
                    &PathNode::Condition(_, b) | &PathNode::Loop(_, b) => {
                        if b {
                            self.path.push(1);
                        } else {
                            self.path.push(0);
                        }
                    },
                    _ => self.path.push(0),
                }

                match node {
                    &PathNode::Condition(l, b) => {
                        let positive = PathNode::Condition(l, true);
                        self.frames.push(Frame::new(positive, Some(old_index.clone())));
                        let negative = PathNode::Condition(l, false);
                        self.frames.push(Frame::new(negative, Some(old_index.clone())));
                    },
                    &PathNode::Loop(l, b) => {
                        let positive = PathNode::Loop(l, true);
                        self.frames.push(Frame::new(positive, Some(old_index.clone())));
                        let negative = PathNode::Loop(l, false);
                        self.frames.push(Frame::new(negative, Some(old_index.clone())));
                    },
                    _ => {
                        self.frames.push(Frame::new(node.clone(), Some(old_index.clone())));
                    }
                }
            }
        }

        self.frames[current_index].set_mapping(name, mapping)
    }

    pub fn set_constant(&mut self, name: String, path: Path, mapping: Mapping) {
        self.set_mapping(name.clone(), path, mapping);
        self.constants.insert(name);
    }

    pub fn change_branch(&mut self) {
        let current = self.path.pop().unwrap();
        self.path.push(current - 1);
    }

    // should only be called when the last frames are Conditions or Loops
    pub fn merge_branches(&mut self) {
        if self.frames.len() == 1 {
            return;
        }

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
            let cause = frame.get_cause();

            for name in &identifiers {
                let old_mapping = self.resolve_optional_identifier(name);
                let mut new_mapping = OptionalMapping::new();

                for (old_path, address) in old_mapping.iter() {
                    let mut new_path = old_path.clone();
                    new_path.add_node(cause.clone());
                    new_mapping.add_mapping(new_path, address.clone());
                }

                new_content.insert(name.clone(), new_mapping);
            }
        }

        // hard coded swap
        {
            let current = self.path.pop().unwrap();
            self.path.push(current + 1);
        }

        // second branch
        {
            let ref frame = self.frames[self.frames.len() - 2];
            let cause = frame.get_cause();

            for name in &identifiers {
                let old_mapping = self.resolve_optional_identifier(name);
                let mut new_mapping = new_content.get_mut(name).unwrap();

                for (old_path, address) in old_mapping.iter() {
                    let mut new_path = old_path.clone();
                    new_path.add_node(cause.clone());
                    new_mapping.add_mapping(new_path, address.clone());
                }
            }
        }

        let _ = self.path.pop();
        let _ = self.frames.pop();
        let _ = self.frames.pop();

        let mut current_index = self.frames.len() - 1;

        current_index -= *self.path.last().unwrap();

        let ref mut current_frame = self.frames[current_index];

        for (name, mapping) in new_content.into_iter() {
            current_frame.set_optional_mapping(name.clone(), mapping)
        }
    }
}
