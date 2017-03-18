use super::{Path, PathNode};
use super::{Mapping, OptionalMapping};
use super::PathID;

use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use std::slice::Iter;
use std::vec::IntoIter;


#[derive(Debug)]
struct Branch {
    content: HashMap<String, OptionalMapping>,
}


impl Branch {
    fn new() -> Branch {
        Branch {
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

    fn set_result(&mut self, mapping: Mapping) {
        self.set_mapping("___result".to_owned(), mapping);
    }

    fn get_result(&self) -> Option<&OptionalMapping> {
        return self.resolve_identifier(&"___result".to_owned());
    }
}


#[derive(Debug)]
struct Frame {
    cause: PathNode,
    branches: Vec<Branch>,
    current: usize,
}

impl Frame {
    fn new(cause: PathNode) -> Self {
        let mut branches = Vec::new();

        let count = match &cause {
            &PathNode::Condition(_ , _, y) | 
            &PathNode::Loop(_, _, y) | 
            &PathNode::Frame(_, _, _, y) => {
                y
            }
            _ => 1,
        };

        for _ in 0..count {
            branches.push(Branch::new());
        }

        Frame {
            cause: cause,
            branches: branches,
            current: 0,
        }
    }

    fn len(&self) -> usize {
        return self.branches.len()
    }

    fn iter(&self) -> Iter<Branch> {
        return self.branches.iter();
    }

    fn into_iter(self) -> IntoIter<Branch> {
        return self.branches.into_iter();
    }

    fn next_branch(&mut self) {
        self.current += 1;
    }

    fn reset_branch_counter(&mut self) {
        self.current = 0;
    }

    fn set_active_branch(&mut self, index: usize) {
        self.current = index;
    }

    fn add_branch(&mut self, branch: Branch) {
        self.branches.push(branch);
    }

    fn resolve_identifier(&self, name: &String) -> Option<&OptionalMapping> {
        return self.branches[self.current].resolve_identifier(name);
    }

    fn set_optional_mapping(&mut self, name: String, mapping: OptionalMapping) {
        self.branches[self.current].set_optional_mapping(name, mapping);
    }

    fn set_mapping(&mut self, name: String, mapping: Mapping) {
        let mut new_mapping = OptionalMapping::new();

        for (path, address) in mapping.iter() {
            new_mapping.add_mapping(path.clone(), Some(address.clone()));
        }

        self.set_optional_mapping(name, new_mapping);
    }


    fn set_result(&mut self, mapping: Mapping) {
        self.set_mapping("___result".to_owned(), mapping);
    }

    fn get_result(&self) -> OptionalMapping {
        let mut mapping = OptionalMapping::new();

        for branch in self.branches.iter() {
            if let Some(m) = branch.resolve_identifier(&"___result".to_owned()) {
                for (path, address) in m.iter() {
                    mapping.add_mapping(path.clone(), address.clone());
                }
            }
        }
        
        return mapping;
    }
}

pub struct Scope {
    frames: Vec<Frame>,
    default: OptionalMapping,
    constants: BTreeSet<String>,
}

impl Scope {
    pub fn new() -> Scope {
        let mut default = OptionalMapping::new();
        default.add_mapping(Path::empty(), None);

        Scope {
            frames: vec![],
            default: default,
            constants: BTreeSet::new(),
        }
    }

    pub fn num_frames(&self) -> usize {
        return self.frames.len();
    }

    pub fn resolve_optional_identifier(&self, name: &String) -> &OptionalMapping {
        if self.frames.len() > 0 {
            let mut index = self.frames.len() - 1;

            while let Some(frame) = self.frames.get(index){
                if let Some(result) = frame.resolve_identifier(name) {
                    return result;
                }

                if index == 0 {
                    break;
                }

                index -= 1;
            } 
        }  
        
        return &self.default;
    }

    fn grow(&mut self, path: &Path, start: usize) {
        for node in path.iter().skip(start) {
            let mut frame = Frame::new(node.clone());
            match node {
                &PathNode::Loop(_ , x, _) |
                &PathNode::Condition(_, x, _) |
                &PathNode::Element(_, x, _) | // should never happen
                &PathNode::Frame(_, _, x, _) => {
                    frame.set_active_branch(x as usize);
                },
                _ => ()
            }
            self.frames.push(frame);
        }
    }

    pub fn set_mapping(&mut self, name: String, path: Path, mapping: Mapping) {
        if self.constants.contains(&name) {
            // todo, throw error
            return;
        }

        let len = self.frames.len();

        if self.frames.len() < path.len() {
            self.grow(&path, len);
        }

        // there should always be one
        if let Some(frame) = self.frames.last_mut() {
            frame.set_mapping(name, mapping);
        }
    }

    pub fn set_result(&mut self, path: Path, mapping: Mapping) {
        self.set_mapping("___result".to_owned(), path, mapping);
    }

    pub fn set_constant(&mut self, name: String, path: Path, mapping: Mapping) {
        self.set_mapping(name.clone(), path, mapping);
        self.constants.insert(name);
    }

    pub fn next_branch(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.next_branch();
        }
    }

    pub fn reset_branch_counter(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.reset_branch_counter();
        }
    }

    pub fn merge_until(&mut self, cutoff: Option<&PathID>) {
        if let Some(cutoff) = cutoff {
            while self.frames.len() > 1 {
                if let Some(frame) = self.frames.last() {
                    let id = frame.cause.get_location();

                    if cutoff >= id {
                        break;
                    } else {
                    }
                } 
                    
                self.merge_branches();
            }
        } else {
            self.merge_branches();
        }
    }

    pub fn discard_branch(&mut self) -> OptionalMapping {
        if self.frames.len() < 2 {
            // shouldn't happen
            return OptionalMapping::new();
        }

        let frame = self.frames.pop().unwrap();

        println!("{:#?}", frame);

        return frame.get_result();
    }

    // only call this when you're certain a single branch was taken
    pub fn lift_branches(&mut self) {
        if self.frames.len() < 2 {
            // should never happen
            return;
        }

        let frame = self.frames.pop().unwrap();

        // copy because frame is about to move
        let cause = frame.cause.clone();

        // lift the mappings
        for (index, branch) in frame.into_iter().enumerate() {
            // todo: maybe better to put usizes in the nodes?
            let index = index as i16; 
            let new_node = match &cause {
                &PathNode::Condition(ref l, _, ref y) => {
                    PathNode::Condition(l.clone(), index, y.clone())
                }
                &PathNode::Loop(ref l, _, ref y) => {
                    PathNode::Loop(l.clone(), index, y.clone())
                }
                &PathNode::Frame(ref l, ref t, _, ref y) => {
                    PathNode::Frame(l.clone(), t.clone(), index, y.clone())
                }
                _ => cause.clone(),
            };

            for (name, mapping) in branch.content.into_iter() {
                let new_mapping = mapping.augment(new_node.clone());

                // there should always be one
                if let Some(new_frame) = self.frames.last_mut() {
                    new_frame.set_optional_mapping(name, new_mapping);
                }
            }            
        }
    }

    // should only be called when the last frames are Conditions or Loops
    fn merge_branches(&mut self) {
        if self.frames.len() < 2 {
            return;
        }

        let cause;
        let mut identifiers;
        let len;

        {
            let frame = self.frames.last().unwrap();    
            identifiers = BTreeSet::new();    

            for branch in frame.iter() {
                for name in branch.content.keys() {
                    identifiers.insert(name.clone());
                }
            }    

            cause = frame.cause.clone();
            len = frame.len();
        }

        self.reset_branch_counter();

        let mut new_content = HashMap::new();

        for i in 0..len {
            let i = i as i16;
            let new_node = match &cause {
                &PathNode::Condition(ref l, _, ref y) => {
                    PathNode::Condition(l.clone(), i, y.clone())
                }
                &PathNode::Loop(ref l, _, ref y) => {
                    PathNode::Loop(l.clone(), i, y.clone())
                }
                &PathNode::Frame(ref l, ref t, _, ref y) => {
                    PathNode::Frame(l.clone(), t.clone(), i, y.clone())
                }
                _ => cause.clone(),
            };

            for name in identifiers.iter() {
                let old_mapping = self.resolve_optional_identifier(name).clone();
                let new_mapping = old_mapping.augment(new_node.clone());

                match new_content.entry(name.clone()) {
                    Entry::Vacant(m) => {
                        m.insert(new_mapping);
                    }
                    Entry::Occupied(mut m) => {
                        let mut old_mapping = m.get_mut();
                        for (path, address) in new_mapping.into_iter() {
                            old_mapping.add_mapping(path, address);
                        }
                    }
                }
            }

            self.next_branch();
        }

        let _ = self.frames.pop();

        if let Some(frame) = self.frames.last_mut() {
            for (name, mapping) in new_content.into_iter() {
                frame.set_optional_mapping(name, mapping)
            }
        }        
    }
}
