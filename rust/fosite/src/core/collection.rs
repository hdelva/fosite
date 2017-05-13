use super::Pointer;
use super::Path;
use super::PathNode;
use super::PathID;

use std::cmp;
use super::Mapping;
use std::slice::Iter;
use std::vec::IntoIter;
use std::collections::btree_map;
use std::collections::LinkedList;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Representant {
    object: Pointer,
    kind: Pointer,
    minimum: Option<usize>,
    maximum: Option<usize>,
}

impl Representant {
    pub fn new(object: Pointer, kind: Pointer, min: Option<usize>, max: Option<usize>) -> Representant {
        Representant {
            object: object,
            kind: kind,
            minimum: min,
            maximum: max,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollectionChunk {
    min_size: Option<usize>,
    max_size: Option<usize>,
    representants: BTreeMap<Path, Representant>,
}

impl CollectionChunk {
    pub fn empty() -> Self {
        CollectionChunk {
            min_size: Some(1),
            max_size: Some(1),
            representants: BTreeMap::new(),
        }
    }

    pub fn add_representant(&mut self, path: Path, repr: Representant)  {
        self.min_size = self.min_size.and_then(|old| repr.minimum.map(|new| cmp::min(old, new)));
        self.max_size = self.max_size.and_then(|old| repr.maximum.map(|new| cmp::max(old, new)));

        self.representants.insert(path, repr);
    }

    pub fn into_iter(self) -> btree_map::IntoIter<Path, Representant> {
        self.representants.into_iter()
    }

    pub fn iter(&self) -> btree_map::Iter<Path, Representant> {
        self.representants.iter()
    }

    pub fn len(&self) -> usize {
        self.representants.len()
    }
}

#[derive(Clone, Debug)]
pub struct CollectionBranch {
    content: Vec<CollectionChunk>,
    min_size: Option<usize>,
    max_size: Option<usize>,
}

impl CollectionBranch {
    pub fn empty() -> Self {
        CollectionBranch {
            content: Vec::new(),
            min_size: Some(0),
            max_size: Some(0),
        }
    }

    pub fn new(content: Vec<CollectionChunk>) -> Self {
        let mut min_size = Some(0);
        let mut max_size = Some(0);

        for chunk in content.iter() {
            min_size = min_size.and_then(|old| chunk.min_size.map(|new| old + new));
            max_size = max_size.and_then(|old| chunk.max_size.map(|new| old + new));
        }

        CollectionBranch { 
            content: content,
            min_size: min_size,
            max_size: max_size,
        }
    }

    pub fn size_range(&self) -> (Option<usize>, Option<usize>) {
        return (self.min_size, self.max_size);
    }

    pub fn is_reliable(&self) -> bool {
        return self.min_size.is_some() && self.max_size.is_some();
    }

    pub fn insert(&mut self, new_chunk: CollectionChunk) {
        self.max_size = self.max_size.and_then(|old| new_chunk.max_size.map(|new| old + new));
        self.min_size = self.min_size.and_then(|old| new_chunk.min_size.map(|new| old + new));

        let mut new_content = vec!(new_chunk.clone());

        for chunk in self.content.iter() {
            new_content.push(chunk.clone());
            new_content.push(new_chunk.clone());
        }

        self.content = new_content;
    }

    pub fn append(&mut self, new_chunk: CollectionChunk) {
        self.min_size = self.min_size.and_then(|old| new_chunk.min_size.map(|new| old + new));
        self.max_size = self.max_size.and_then(|old| new_chunk.max_size.map(|new| old + new));

        self.content.push(new_chunk);
    }

    pub fn prepend(&mut self, new_chunk: CollectionChunk) {
        self.min_size = self.min_size.and_then(|old| new_chunk.min_size.map(|new| old + new));
        self.max_size = self.max_size.and_then(|old| new_chunk.max_size.map(|new| old + new));

        self.content.insert(0, new_chunk);
    }

    fn first_combinations(&self, n: i16) -> Vec<LinkedList<Mapping>> {
        let result = linearize(n as usize, &self.content, false);
        // todo, make efficient pls
        return result.iter().cloned().map(|x| x.iter().cloned().rev().collect()).collect();
    }

    fn last_combinations(&self, n: i16) -> Vec<LinkedList<Mapping>> {
        let result = linearize(n as usize, &self.content, true);
        return result;
    }

    pub fn get_element(&self, n: i16) -> Mapping {
        let mut result = Mapping::new();

        if n < 0 {
            for possibility in self.last_combinations(-n) {
                // get the first mapping of the the last n for element -n
                for &(ref path, ref address) in possibility.front().unwrap().iter() {
                    result.add_mapping(path.clone(), address.clone());
                }
            }
        } else {
            // get the last mapping of the the first n for element n
            for possibility in self.first_combinations(n) {
                for &(ref path, ref address) in possibility.back().unwrap().iter() {
                    result.add_mapping(path.clone(), address.clone());
                }
            }
        }

        return result;
    }

    pub fn get_any_element(&self) -> Mapping {
        let mut result = Mapping::new();
        for chunk in self.content.iter() {
            for (path, repr) in chunk.iter() {
                result.add_mapping(path.clone(), repr.object.clone());
            }
        }
        return result;
    }

    pub fn get_types(&self) -> BTreeSet<Pointer> {
        let mut result = BTreeSet::new();
        for chunk in self.content.iter() {
            for (_, repr) in chunk.iter() {
                result.insert(repr.kind.clone());
            }
        }
        return result;
    }

    pub fn get_first_n(&self, n: i16) -> Vec<Mapping> {
        let mut result = Vec::new();

        let mut combinations = self.first_combinations(n);
        for _ in 0..n {
            let mut element = Mapping::new();
            for mut combination in combinations.iter_mut() {
                let mapping = combination.pop_front();
                if let Some(mapping) = mapping {
                    for (path, address) in mapping.into_iter() {
                        element.add_mapping(path, address);
                    }
                } else {
                    break;
                }
            }
            result.push(element);
        }

        return result;
    }

    pub fn get_last_n(&self, n: i16) -> Vec<Mapping> {
        let mut result = Vec::new();

        let mut combinations = self.last_combinations(n);
        for _ in 0..n {
            let mut element = Mapping::new();
            for mut combination in combinations.iter_mut() {
                let mapping = combination.pop_front();
                if let Some(mapping) = mapping {
                    for (path, address) in mapping.into_iter() {
                        element.add_mapping(path, address);
                    }
                } else {
                    break;
                }
            }
            result.push(element);
        }

        return result;
    }

    pub fn slice(&self, start: i16, end: i16) -> CollectionBranch {
        let mut min_counts: BTreeMap<Mapping, usize> = BTreeMap::new();
        let mut max_counts: BTreeMap<Mapping, usize> = BTreeMap::new();

        let head = self.first_combinations(start);

        for possibilities in head {
            // todo, make pls: HashMap<&Mapping, i32>
            let mut pls = BTreeMap::new();
            for mapping in possibilities.iter() {
                *pls.entry(mapping.clone()).or_insert(0) += 1;
            }

            for (mapping, count) in pls.into_iter() {
                match min_counts.entry(mapping.clone()) {
                    btree_map::Entry::Occupied(mut c) => {
                        *c.get_mut() = cmp::min(c.get().clone(), count);
                    },
                    btree_map::Entry::Vacant(c) => {
                        c.insert(count);
                    }
                }

                match max_counts.entry(mapping) {
                    btree_map::Entry::Occupied(mut c) => {
                        *c.get_mut() = cmp::max(c.get().clone(), count);
                    },
                    btree_map::Entry::Vacant(c) => {
                        c.insert(count);
                    }
                }
            }
        }

        let tail = self.last_combinations(end);

        for possibility in tail {
            let mut pls = BTreeMap::new();
            for mapping in possibility.into_iter() {
                *pls.entry(mapping.clone()).or_insert(0) += 1;
            }

            for (mapping, count) in pls.into_iter() {
                match min_counts.entry(mapping.clone()) {
                    btree_map::Entry::Occupied(mut c) => {
                        *c.get_mut() = cmp::min(c.get().clone(), count);
                    },
                    btree_map::Entry::Vacant(c) => {
                        c.insert(count);
                    }
                }

                match max_counts.entry(mapping) {
                    btree_map::Entry::Occupied(mut c) => {
                        *c.get_mut() = cmp::max(c.get().clone(), count);
                    },
                    btree_map::Entry::Vacant(c) => {
                        c.insert(count);
                    }
                }
            }
        }

        let mut new_content = Vec::new();

        // todo, this is an inaccurate way of updating the counts
        // we'd need chunk information to update the representants of each chunk 
        // every chunks get updated according to the whole branch's changes at the moment
        //
        // slicing a, *b, c = [x, x, x] is going to do weird things
        for chunk in self.content.iter() {
            let mut new_chunk = CollectionChunk::empty();
            for (path, repr) in chunk.iter() {
                
                // todo, this seems really inefficient
                let dummy_mapping = Mapping::simple(path.clone(), repr.object.clone());
                let delta_min = min_counts.get(&dummy_mapping);
                let delta_max = max_counts.get(&dummy_mapping);

                let new_min = repr.minimum.map(|old| old - *delta_min.unwrap_or(&0));
                let new_max = repr.maximum.map(|old| old - *delta_max.unwrap_or(&0));

                if new_max.unwrap_or(1) > 0 {
                    let new_repr = Representant::new(repr.object.clone(), repr.kind.clone(), new_min, new_max);
                    new_chunk.add_representant(path.clone(), new_repr);
                }
            }

            if new_chunk.len() > 0 {
                new_content.push(new_chunk);
            }
        }

        return CollectionBranch::new(new_content);
    }

    pub fn concatenate(&mut self, other: CollectionBranch) {
        for chunk in other.content.into_iter() {
            self.append(chunk);
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionMapping {
    path: Path,
    branch: CollectionBranch,
}

impl CollectionMapping {
    fn new( path: Path, branch: CollectionBranch) -> Self {
        CollectionMapping {
            branch: branch,
            path: path,
        }
    }

    fn augment(mut self, node: PathNode) -> CollectionMapping {
        self.path.add_node(node.clone());

        return CollectionMapping::new(self.path, self.branch);
    }
}

#[derive(Debug, Clone)]
struct Branch {
    content: Vec<CollectionMapping>,
}

impl Branch {  
    pub fn new(content: Vec<CollectionMapping>) -> Self {
        Branch {
            content: content,
        }
    }

    pub fn get_content(&self) -> &Vec<CollectionMapping> {
        return &self.content;
    }

    pub fn iter(&self) -> Iter<CollectionMapping> {
        return self.content.iter();
    }

    pub fn into_iter(self) -> IntoIter<CollectionMapping> {
        return self.content.into_iter();
    }

    pub fn add_mapping(&mut self, mapping: CollectionMapping) {
        self.content.push(mapping);
    }

    pub fn set_content(&mut self, content: Vec<CollectionMapping>) {
        self.content = content;
    }

    // interface pass through
    pub fn size_range(&self) -> Vec<(Path, Option<usize>, Option<usize>)> {
        let mut result = Vec::new();
        for mapping in self.content.iter() {
            let &CollectionMapping {ref path, ref branch} = mapping;
            let (min, max) = branch.size_range();
            result.push( (path.clone(), min, max) );
        }
        return result;
    }

    pub fn is_reliable(&self) -> Vec<(Path, bool)> {
        let mut result = Vec::new();
        for mapping in self.content.iter() {
            let &CollectionMapping {ref path, ref branch} = mapping;
            let rel = branch.is_reliable();
            result.push( (path.clone(), rel) );
        }
        return result;
    }

    pub fn insert(&mut self, element: CollectionChunk) {
        for mapping in self.content.iter_mut() {
            let &mut CollectionMapping {ref mut branch, ..} = mapping;
            branch.insert(element.clone());
        }
    }

    pub fn define(&mut self, definition: Vec<CollectionChunk>) {
        let new_branch = CollectionBranch::new(definition);
        let mapping = CollectionMapping::new(Path::empty(), new_branch);
        self.content = vec!(mapping);
    }

    pub fn append(&mut self, element: CollectionChunk) {
        for mapping in self.content.iter_mut() {
            let &mut CollectionMapping {ref mut branch, ..} = mapping;
            branch.append(element.clone());
        }
    }

    pub fn prepend(&mut self, element: CollectionChunk) {
        for mapping in self.content.iter_mut() {
            let &mut CollectionMapping {ref mut branch, ..} = mapping;
            branch.prepend(element.clone());
        }
    }

    pub fn get_element(&self, n: i16, node: &PathID) -> Mapping {
        let mut result = Mapping::new();
        let mut count = 0;
        for coll_mapping in self.content.iter() {
            let &CollectionMapping {ref branch, ..} = coll_mapping;
            
            let possibilities = branch.get_element(n);
            let total = possibilities.len().clone() as i16;
            for (path, address) in possibilities.into_iter() {
                let mut new_path = path.clone();

                // add the element's path
                new_path.merge_into(path);

                // add the possibility counter 
                let new_node = PathNode::Element(node.clone(), count, total); 
                new_path.add_node(new_node);

                // combine it all into a new mapping
                result.add_mapping(new_path, address.clone());
                count += 1;
            }
        }
        return result;
    }

    pub fn get_any_element(&self, node: &PathID) -> Mapping {
        let mut result = Mapping::new();
        let mut count = 0;
        for coll_mapping in self.content.iter() {
            let &CollectionMapping {ref path, ref branch} = coll_mapping;

            let possibilities = branch.get_any_element();
            let total = possibilities.len().clone() as i16;
            
            for (element_path, address) in possibilities.into_iter() {
                let mut new_path = path.clone();

                // add the element's path
                new_path.merge_into(element_path);

                // add the possibility counter 
                let new_node = PathNode::Element(node.clone(), count, total); 
                new_path.add_node(new_node);

                // combine it all into a new mapping
                result.add_mapping(new_path, address.clone());
                count += 1;
            }
        }
        return result;
    }

    pub fn get_types(&self) -> BTreeSet<Pointer> {
        let mut result = BTreeSet::new();
        for coll_mapping in self.content.iter() {
            let &CollectionMapping {ref branch, ..} = coll_mapping;

            let mut possibilities = branch.get_types();

            result.append(&mut possibilities);
        }
        return result;
    }

    pub fn get_first_n(&self, n: i16, node: &PathID) -> Vec<Mapping> {
        let mut result = Vec::new();
        for coll_mapping in self.content.iter() {
            let &CollectionMapping {ref path, ref branch} = coll_mapping;

            for (index, mapping) in branch.get_first_n(n).into_iter().enumerate() {
                if result.len() <= index {
                    result.push(Mapping::new());
                }

                let ref mut new_mapping = result[index]; 
                let mut count = new_mapping.len() as i16;

                for (element_path, address) in mapping.into_iter() {
                    let mut new_path = path.clone();

                    // add the element's path
                    new_path.merge_into(element_path);

                    // add the possibility counter 
                    let new_node = PathNode::Element(node.clone(), count, 0); 
                    new_path.add_node(new_node);

                    // combine it all into a new mapping
                    new_mapping.add_mapping(new_path, address.clone());
                    count += 1;
                }
            }
        }
        return result;
    }

    pub fn get_last_n(&self, n: i16, node: &PathID) -> Vec<Mapping> {
        let mut result = Vec::new();
        for coll_mapping in self.content.iter() {
            let &CollectionMapping {ref path, ref branch} = coll_mapping;

            for (index, mapping) in branch.get_last_n(n).into_iter().enumerate() {
                if result.len() <= index {
                    result.push(Mapping::new());
                }

                let ref mut new_mapping = result[index]; 
                let mut count = new_mapping.len() as i16;

                for (element_path, address) in mapping.into_iter() {
                    let mut new_path = path.clone();

                    // add the element's path
                    new_path.merge_into(element_path);

                    // add the possibility counter 
                    let new_node = PathNode::Element(node.clone(), count, 0); 
                    new_path.add_node(new_node);

                    // combine it all into a new mapping
                    new_mapping.add_mapping(new_path, address.clone());
                    count += 1;
                }
            }
        }

        return result;
    }

    pub fn slice(&self, start: i16, end: i16) -> Vec<(Path, CollectionBranch)> {
        let mut result = Vec::new();
        for coll_mapping in self.content.iter() {
            let &CollectionMapping {ref path, ref branch} = coll_mapping;
            let new_branch = branch.slice(start, end);
            result.push( (path.clone(), new_branch));
        }
        return result;
    }

}

#[derive(Debug, Clone)]
pub struct Frame {
    branches: Vec<Branch>,
    cause: PathNode,
    current: usize,
}

impl Frame {
    fn new(cause: PathNode) -> Self {
        Frame {
            cause: cause,
            branches: Vec::new(),
            current: 0,
        }
    }

    // frame things
    fn set_content(&mut self, content: Vec<CollectionMapping>) {
        self.branches[self.current].set_content(content);
    }

    fn get_content(&self) -> &Vec<CollectionMapping> {
        self.branches[self.current].get_content()
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

    pub fn size_range(&self) -> Vec<(Path, Option<usize>, Option<usize>)> {
        self.branches[self.current].size_range()
    }

    pub fn is_reliable(&self) -> Vec<(Path, bool)> {
        self.branches[self.current].is_reliable()
    }

    pub fn insert(&mut self, element: CollectionChunk) {
        self.branches[self.current].insert(element)
    }

    pub fn define(&mut self, definition: Vec<CollectionChunk>) {
        self.branches[self.current].define(definition)
    }

    pub fn append(&mut self, element: CollectionChunk) {
        self.branches[self.current].append(element)
    }

    pub fn prepend(&mut self, element: CollectionChunk) {
        self.branches[self.current].prepend(element)
    }

    pub fn get_element(&self, n: i16, node: &PathID) -> Mapping {
        self.branches[self.current].get_element(n, node)
    }

    pub fn get_any_element(&self, node: &PathID) -> Mapping {
        self.branches[self.current].get_any_element(node)
    }

    pub fn get_types(&self) -> BTreeSet<Pointer> {
        self.branches[self.current].get_types()
    }

    pub fn get_first_n(&self, n: i16, node: &PathID) -> Vec<Mapping> {
        self.branches[self.current].get_first_n(n, node)
    }

    pub fn get_last_n(&self, n: i16, node: &PathID) -> Vec<Mapping> {
        self.branches[self.current].get_last_n(n, node)
    }

    pub fn slice(&self, start: i16, end: i16) -> Vec<(Path, CollectionBranch)> {
        self.branches[self.current].slice(start, end)
    }
}

#[derive(Clone, Debug)]
pub struct Collection {
    frames: Vec<Frame>,
}

impl Collection {
    pub fn new() -> Collection {
        let mut frame = Frame::new(PathNode::Frame(vec!(0), None, 0, 1));
        frame.add_branch(Branch::new(Vec::new()));
        Collection {
            frames: vec![frame],
        }
    }

    pub fn concatenate(&self, other: &Collection) -> Collection {
        let mut new = Collection::new();
        let mut new_content = Vec::new();

        if let Some(frame) = self.frames.last(){
            if let Some(other_frame) = other.frames.last() {
                for mapping in frame.branches[frame.current].iter() {
                    for other_mapping in other_frame.branches[other_frame.current].iter() {
                        let mut new_path = mapping.path.clone();
                        new_path.merge_into(other_mapping.path.clone());
                        let mut new_branch = mapping.branch.clone();
                        new_branch.concatenate(other_mapping.branch.clone());

                        new_content.push((new_path, new_branch));
                    }
                }  
            }
        }

        new.set_content(new_content);
        return new;
    }

    pub fn set_content(&mut self, content: Vec<(Path, CollectionBranch)>) {
        let mut mappings = Vec::new();
        for (path, branch) in content.into_iter() {
            mappings.push(CollectionMapping::new(path, branch));
        }

        if let Some(frame) = self.current_frame_mut(Path::empty()) {
            frame.set_content(mappings);
        }
    }

    // collection things

    pub fn num_frames(&self) -> usize {
        return self.frames.len();
    }

    pub fn grow(&mut self, path: &Path, start: usize) {
        for node in path.iter().skip(start) {
            let current_content = self.frames.last().unwrap().get_content().clone();
            let mut frame = Frame::new(node.clone());
            match node {
                &PathNode::Condition(_, x, y) |
                &PathNode::Element(_, x, y) | // should never happen
                &PathNode::Frame(_, _, x, y) => {
                    for _ in 0..y {
                        frame.add_branch(Branch::new(current_content.clone()));
                    }
                    frame.set_active_branch(x as usize);
                },
                _ => {
                    // probably a loop then
                    frame.add_branch(Branch::new(current_content.clone()));
                }
            }
            self.frames.push(frame);
        }
    }

    pub fn current_frame_mut(&mut self, path: Path) -> Option<&mut Frame> {
        let len = self.frames.len();

        if self.frames.len() < path.len() {
            self.grow(&path, len);
        }

        self.frames.last_mut()
    }

    pub fn next_branch(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.next_branch()
        }
    }

    pub fn reset_branch_counter(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.reset_branch_counter()
        }
    }

    // collections do the same thing for lifting as for merging
    // 
    pub fn lift_branches(&mut self) {
        self.merge_branches();
    }

    pub fn merge_until(&mut self, cutoff: Option<&PathID>) {
        if let Some(cutoff) = cutoff {
            while self.frames.len() > 1 {
                let mut b = false;

                if let Some(frame) = self.frames.last() {
                    let id = frame.cause.get_location();
                    b = cutoff > id;
                } 

                if b {
                    break;
                }
                    
                self.merge_branches();
            }
        } else {
            self.merge_branches();
        }
    }

    pub fn merge_branches(&mut self) {
        if self.frames.len() < 1 {
            return;
        }

        let frame = self.frames.pop().unwrap();

        // copy because frame is going to get moved
        let cause = frame.cause.clone();

        let mut new_content = Vec::new();

        for (index, branch) in frame.into_iter().enumerate() {
            let index = index as i16;
            let new_node = match &cause {
                &PathNode::Condition(ref l, _, ref y) => {
                    PathNode::Condition(l.clone(), index, y.clone())
                }
                &PathNode::Frame(ref l, ref t, _, ref y) => {
                    PathNode::Frame(l.clone(), t.clone(), index, y.clone())
                }
                _ => cause.clone(),
            };

            for collection_mapping in branch.into_iter() {
                let new_mapping = collection_mapping.augment(new_node.clone());
                new_content.push(new_mapping);
            }
        }

        if let Some(frame) = self.frames.last_mut() {
            frame.set_content(new_content);
        } 
    }

    // interface pass-through things
    pub fn size_range(&self) -> Vec<(Path, Option<usize>, Option<usize>)> {
        if let Some(frame) = self.frames.last() {
            frame.size_range()
        } else {
            panic!("No frames in this collection")
        }
    }

    pub fn is_reliable(&self) -> Vec<(Path, bool)> {
        if let Some(frame) = self.frames.last() {
            frame.is_reliable()
        } else {
            panic!("No frames in this collection")
        }
    }

    pub fn get_element(&self, n: i16, node: &PathID) -> Mapping {
        if let Some(frame) = self.frames.last() {
            frame.get_element(n, node)
        }  else {
            panic!("No frames in this collection")
        }
    }

    pub fn get_any_element(&self, node: &PathID) -> Mapping {
        if let Some(frame) = self.frames.last() {
            frame.get_any_element(node)
        } else {
            panic!("No frames in this collection")
        }
    }

    pub fn get_types(&self) -> BTreeSet<Pointer> {
        if let Some(frame) = self.frames.last() {
            frame.get_types()
        } else {
            panic!("No frames in this collection")
        }
    }

    pub fn get_first_n(&self, n: i16, node: &PathID) -> Vec<Mapping> {
        if let Some(frame) = self.frames.last() {
            frame.get_first_n(n, node)
        } else {
            panic!("No frames in this collection")
        }
    }

    pub fn get_last_n(&self, n: i16, node: &PathID) -> Vec<Mapping> {
        if let Some(frame) = self.frames.last() {
            frame.get_last_n(n, node)
        } else {
            panic!("No frames in this collection")
        }
    }

    pub fn slice(&self, start: i16, end: i16) -> Vec<(Path, CollectionBranch)> {
        if let Some(frame) = self.frames.last() {
            frame.slice(start, end)
        } else {
            panic!("No frames in this collection")
        }
    }

    pub fn insert(&mut self, element: CollectionChunk, path: Path) {
        if let Some(frame) = self.current_frame_mut(path) {
            frame.insert(element)
        } else {
            panic!("No frames in this collection")
        }
    }

    pub fn define(&mut self, definition: Vec<CollectionChunk>, path: Path) {
        if let Some(frame) = self.current_frame_mut(path) {
            frame.define(definition)
        } else {
            panic!("No frames in this collection")
        }
    }

    pub fn append(&mut self, element: CollectionChunk, path: Path) {
        if let Some(frame) = self.current_frame_mut(path) {
            frame.append(element)
        } else {
            panic!("No frames in this collection")
        }
    }

    pub fn prepend(&mut self, element: CollectionChunk, path: Path) {
        if let Some(frame) = self.current_frame_mut(path) {
            frame.prepend(element)
        } else {
            panic!("No frames in this collection")
        }
    }
}

fn linearize(n: usize,
                chunks: &[CollectionChunk],
                reverse: bool)
                -> Vec<LinkedList<Mapping>> {
    let chunk;
    let next_chunk;

    if n <= 0 || chunks.len() == 0 {
        // safety net, should never be true
        return vec![];
    }

    if reverse {
        chunk = chunks.last();
        next_chunk = &chunks[..chunks.len() - 1];
    } else {
        chunk = chunks.first();
        next_chunk = &chunks[1..];
    }

    match chunk {
        Some(chunk) => {
            let mut result = Vec::new();

            'representants:
            for (path, repr) in chunk.iter() {
                let mut pls: LinkedList<Mapping> = LinkedList::new();

                for _ in 0..repr.minimum.unwrap_or(0 as usize) {
                    pls.push_front(Mapping::simple(path.clone(), repr.object.clone()));
                    if pls.len() >= n as usize {
                        // just adding the minimum amount of elements in enough 
                        // no need for recursion in this case
                        // move on to the next representant in this chunk
                        result.push(pls);
                        continue 'representants;
                    }
                }

                // minimum has been reached, we can now move to the next chunk
                // get all sequences of length `n - current_length`
                let intermediate = linearize(n - pls.len(), next_chunk, reverse);

                // add the current stuff
                // length becomes `n` now
                for mut sequence in intermediate.into_iter() {
                    sequence.append(&mut pls.clone());
                    result.push(sequence);
                }

                // or rather than going to the next chunk,
                // add more until we reach the maximum number
                for _ in repr.minimum.unwrap_or(0)..repr.maximum.unwrap_or(n as usize) {
                    pls.push_front(Mapping::simple(path.clone(), repr.object.clone()));

                    if pls.len() >= n as usize {
                        // maximum length has been reached 
                        // no need for recursion in this case
                        // move on to the next representant in this chunk
                        result.push(pls);
                        continue 'representants;
                    } 

                    let intermediate = linearize(n - pls.len(), next_chunk, reverse);
                    
                    // add the current stuff
                    // length becomes `n` now
                    for mut sequence in intermediate.into_iter() {
                        sequence.append(&mut pls.clone());
                        result.push(sequence);
                    }
                }
            }
            return result;
        }
        _ => panic!("not enough elements to unpack"),
    }
}
