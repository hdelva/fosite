use super::{Path, PathNode};
use super::{Mapping, OptionalMapping};

use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use std::slice::{Iter, IterMut};
use std::vec::IntoIter;


#[derive(Debug, Clone)]
struct Branch {
    content: HashMap<String, OptionalMapping>,
}


impl Branch {
    fn new() -> Branch {
        Branch {
            content: HashMap::new(),
        }
    }

    pub fn add_optional_mapping(&mut self, name: String, mapping: OptionalMapping) {
        match self.content.entry(name) {
            Entry::Vacant(v) => {
                v.insert(mapping);
            },
            Entry::Occupied(mut o) => {
                let mut b = o.get_mut();
                for (path, address) in mapping.into_iter() {
                    b.add_mapping(path, address);
                }
            }
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
            new_mapping
            .add_mapping(path.clone(), Some(address.clone()));
        }

        self.set_optional_mapping(name, new_mapping);
    }

    fn set_result(&mut self, mapping: Mapping) {
        self.set_mapping("___result".to_owned(), mapping);
    }

    fn get_result(&self) -> Option<&OptionalMapping> {
        return self.resolve_identifier(&"___result".to_owned());
    }

    fn contains_mapping(&self, name: &String) -> bool {
        return self.content.contains_key(name);
    }
}

#[derive(Debug, Clone)]
struct StatisChamber {
    content: HashMap<Path, Branch>,
}

impl StatisChamber {
    pub fn new() -> Self {
        StatisChamber {
            content: HashMap::new(),
        }
    }

    pub fn thaw(self) -> HashMap<String, OptionalMapping> {
        let mut result = HashMap::new();

        for (path, branch) in self.content.into_iter() {
            for (name, mapping) in branch.content.into_iter() {
                for (mapping_path, address) in mapping.into_iter() {
                    let mut new_path = path.clone();
                    new_path.merge_into(mapping_path.clone());

                    match result.entry(name.clone()) {
                        Entry::Occupied(mut o) => {
                            let b: &mut OptionalMapping = o.get_mut();
                            b.add_mapping(new_path, address);
                        },
                        Entry::Vacant(v) => {
                            let mut b = OptionalMapping::new();
                            b.add_mapping(new_path, address); 
                            v.insert(b);
                        }
                    }
                }
            }
        } 

        return result;
    }

    pub fn fill(&mut self, name: String, mapping: OptionalMapping) {
        for (_, b) in self.content.iter_mut() {
            if !b.contains_mapping(&name) {
                b.set_optional_mapping(name.clone(), mapping.clone());
            }
        }
    }

    pub fn add_mapping(&mut self, path: Path, name: String, mapping: OptionalMapping) {
        match self.content.entry(path) {
            Entry::Vacant(v) => {
                let mut b = Branch::new();
                b.set_optional_mapping(name, mapping);
                v.insert(b);
            },
            Entry::Occupied(mut o) => {
                let mut b = o.get_mut();
                b.set_optional_mapping(name, mapping);
            }
        }
    }

    pub fn grow(&mut self, cause: PathNode) {
        let mut new_content = HashMap::new();

        for (path, b) in self.content.iter() {
            let mut new_path = path.clone();
            new_path.add_node(cause.clone());
            new_content.insert(new_path, b.clone());
        }

        self.content = new_content;
    }
}

#[derive(Debug, Clone)]
struct SubFrame {
    content: Branch,
    frozen_loop: StatisChamber,
    frozen_function: StatisChamber,
}

impl SubFrame {
    pub fn new() -> Self {
        SubFrame {
            content: Branch::new(),
            frozen_loop: StatisChamber::new(),
            frozen_function: StatisChamber::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct Frame {
    cause: PathNode,
    branches: Vec<SubFrame>,
    current: usize,
}

impl Frame {
    fn new(cause: PathNode) -> Self {
        let mut branches = Vec::new();

        let count = match &cause {
            &PathNode::Condition(_ , _, y) | 
            &PathNode::Frame(_, _, _, y) => {
                y
            }
            _ => 1,
        };

        for _ in 0..count {
            branches.push(SubFrame::new());
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

    fn iter(&self) -> Iter<SubFrame> {
        return self.branches.iter();
    }

    fn iter_mut(&mut self) -> IterMut<SubFrame> {
        return self.branches.iter_mut();
    }

    fn into_iter(self) -> IntoIter<SubFrame> {
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

    fn resolve_identifier(&self, name: &String) -> Option<&OptionalMapping> {
        return self.branches[self.current].content.resolve_identifier(name);
    }

    fn set_optional_mapping(&mut self, name: String, mapping: OptionalMapping) {
        let old_mapping;
        if let Some(pls) = self.resolve_identifier(&name) {
            old_mapping = pls.clone();
        } else {
            let mut pls2 = OptionalMapping::new();
            pls2.add_mapping(Path::empty(), None);
            old_mapping = pls2;
        }

        self.branches[self.current].frozen_loop.fill(name.clone(), old_mapping.clone());
        self.branches[self.current].frozen_function.fill(name.clone(), old_mapping);

        self.branches[self.current].content.set_optional_mapping(name, mapping);
    }

    fn add_optional_mapping(&mut self, name: String, mapping: OptionalMapping) {
        self.branches[self.current].content.add_optional_mapping(name, mapping);
    }

    fn set_loop_statis(&mut self, path: Path, branch: Branch) {
        self.branches[self.current].frozen_loop.content.insert(path, branch);
    }

    fn set_function_statis(&mut self, path: Path, branch: Branch) {
        self.branches[self.current].frozen_function.content.insert(path, branch);
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
            if let Some(m) = branch.content.resolve_identifier(&"___result".to_owned()) {
                for (path, address) in m.iter() {
                    mapping.add_mapping(path.clone(), address.clone());
                }
            }
        }
        
        return mapping;
    }
}

#[derive(Debug)]
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
        let mut new_mapping = OptionalMapping::new();

        for (path, address) in mapping.into_iter() {
            new_mapping.add_mapping(path, Some(address));
        }

        self.set_optional_mapping(name, path, new_mapping);
    }

    pub fn set_optional_mapping(&mut self, name: String, path: Path, mapping: OptionalMapping) {
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
            frame.set_optional_mapping(name, mapping);
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

    pub fn discard_function(mut self) -> OptionalMapping {
        if let Some(frame) = self.frames.pop() {
            return frame.get_result();
        }

        return self.default.clone();
    }

    pub fn merge_branches(&mut self, hide_as_loop: Vec<Option<bool>>) {
        if self.frames.len() < 2 {
            return;
        }

        let (new_content, mut new_loop, mut new_function) = self.prepare_merge(hide_as_loop);

        // remember all the old hides
        if let Some(old_frame) = self.frames.pop() {
            for subframe in old_frame.branches.into_iter() {
                for (path, branch) in subframe.frozen_loop.content.into_iter() {
                    new_loop.content.insert(path, branch);
                }

                for (path, branch) in subframe.frozen_function.content.into_iter() {
                    new_function.content.insert(path, branch);
                }
            }
            
        } 

        if let Some(frame) = self.frames.last_mut() {
            // transfer the hidden mappings
            for (name, mapping) in new_content.content.into_iter() {
                frame.set_optional_mapping(name, mapping);
            }

            for (path, branch) in new_loop.content.into_iter() {
                frame.set_loop_statis(path, branch);
            }

            for (path, branch) in new_function.content.into_iter() {
                frame.set_function_statis(path, branch);
            }
        }   
    }

    pub fn merge_loop(&mut self) {
        if self.frames.len() < 2 {
            return;
        }

        let (new_content, mut new_loop, mut new_function) = self.prepare_merge(vec!());

        if let Some(old_frame) = self.frames.pop() {
            for subframe in old_frame.branches.into_iter() {
                for (path, branch) in subframe.frozen_loop.content.into_iter() {
                    new_loop.content.insert(path, branch);
                } 

                for (path, branch) in subframe.frozen_function.content.into_iter() {
                    new_function.content.insert(path, branch);
                }  
            }
        } 

        if let Some(frame) = self.frames.last_mut() {
            // transfer the hidden mappings
            for (name, mapping) in new_content.content.into_iter() {
                frame.set_optional_mapping(name, mapping);
            }

            for (path, branch) in new_function.content.into_iter() {
                frame.set_function_statis(path, branch);
            }

            for (name, mapping) in new_loop.thaw().into_iter() {
                frame.add_optional_mapping(name, mapping);
            }
        } 
    }

    pub fn merge_function(&mut self) {
        if self.frames.len() < 2 {
            return;
        }

        let (new_content, mut new_loop, mut new_function) = self.prepare_merge(vec!());

        if let Some(old_frame) = self.frames.pop() {
            for subframe in old_frame.branches.into_iter() {
                for (path, branch) in subframe.frozen_loop.content.into_iter() {
                    new_loop.content.insert(path, branch);
                } 

                for (path, branch) in subframe.frozen_function.content.into_iter() {
                    new_function.content.insert(path, branch);
                }  
            }
        } 

        if let Some(frame) = self.frames.last_mut() {
            // transfer the hidden mappings
            for (name, mapping) in new_content.content.into_iter() {
                frame.set_optional_mapping(name, mapping);
            }

            for (name, mapping) in new_loop.thaw().into_iter() {
                frame.add_optional_mapping(name, mapping);
            }

            for (name, mapping) in new_function.thaw().into_iter() {
                frame.add_optional_mapping(name, mapping);
            }
        } 
    }

    fn prepare_merge(&mut self, hide_as_loop: Vec<Option<bool>>) 
        -> (Branch, StatisChamber, StatisChamber) {
            
        let cause;
        let mut merge_identifiers;
        let len;

        {
            let mut frame = self.frames.last_mut().unwrap();   

            cause = frame.cause.clone();
            len = frame.len();

            merge_identifiers = BTreeSet::new();    

            for (i, subframe) in frame.iter_mut().enumerate() {
                let i = i as i16;
                let new_node = match &cause {
                    &PathNode::Condition(ref l, _, ref y) => {
                        PathNode::Condition(l.clone(), i, y.clone())
                    }
                    &PathNode::Frame(ref l, ref t, _, ref y) => {
                        PathNode::Frame(l.clone(), t.clone(), i, y.clone())
                    }
                    _ => cause.clone(),
                };

                for name in subframe.content.content.keys() {
                    merge_identifiers.insert(name.clone());
                }

                subframe.frozen_loop.grow(new_node);
            }    
        }

        self.reset_branch_counter();

        let mut new_content = Branch::new();
        let mut new_loop_freeze = StatisChamber::new();
        let mut new_function_freeze = StatisChamber::new();

        for i in 0..len {
            let i = i as i16;
            let new_node = match &cause {
                &PathNode::Condition(ref l, _, ref y) => {
                    PathNode::Condition(l.clone(), i, y.clone())
                }
                &PathNode::Frame(ref l, ref t, _, ref y) => {
                    PathNode::Frame(l.clone(), t.clone(), i, y.clone())
                }
                _ => cause.clone(),
            };

            let iter = merge_identifiers.iter();

            for name in iter {
                let old_mapping = self.resolve_optional_identifier(name).clone();
                let new_mapping = old_mapping.augment(new_node.clone());

                let entry;
                if let &Some(ref b) = hide_as_loop.get(i as usize).unwrap_or(&None) {
                    let mut p = Path::empty();
                    p.add_node(cause.clone());
                    if *b {
                        let pls = new_loop_freeze.content.entry(p).or_insert(Branch::new());
                        entry = pls.content.entry(name.clone());
                    } else {
                        let pls = new_function_freeze.content.entry(p).or_insert(Branch::new());
                        entry = pls.content.entry(name.clone());
                    }
                } else {
                    entry = new_content.content.entry(name.clone());
                }

                match entry {
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

        return (new_content, new_loop_freeze, new_function_freeze);
    } 
}
