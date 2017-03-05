use super::Pointer;
use super::Path;
use super::PathNode;
use super::GastID;

use std::collections::hash_map::Entry;
use std::cmp;
use super::Mapping;
use std::iter::FromIterator;
use std::slice::Iter;
use std::vec::IntoIter;
use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::btree_map;
use std::collections::LinkedList;
use std::collections::BTreeMap;

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
            min_size: Some(0),
            max_size: Some(0),
            representants: BTreeMap::new(),
        }
    }

    pub fn add_representant(&mut self, path: Path, repr: Representant)  {
        self.min_size = self.min_size.and_then(|old| repr.minimum.map(|new| old + new));
        self.max_size = self.max_size.and_then(|old| repr.maximum.map(|new| old + new));

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
        return linearize(n as usize, &self.content, false);
    }

    fn last_combinations(&self, n: i16) -> Vec<LinkedList<Mapping>> {
        let mut result = linearize(n as usize, &self.content, true);
        result.reverse();
        return result;
    }

    pub fn get_element(&self, n: i16) -> Mapping {
        let mut result = Mapping::new();
        
        if n < 0 {
            for possibility in self.last_combinations(-n) {
                // get the first mapping of the the last n for element -n
                for (path, address) in possibility.front().unwrap().iter() {
                    result.add_mapping(path.clone(), address.clone());
                }
            }
        } else {
            // get the last mapping of the the first n for element n
            for possibility in self.first_combinations(n) {
                for (path, address) in possibility.front().unwrap().iter() {
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

    pub fn get_first_n(&self, n: i16) -> Vec<Mapping> {
        let mut result = Vec::new();

        let mut combinations = self.first_combinations(n);
        for i in 0..n {
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
        for i in 0..n {
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
}

#[derive(Debug, Clone)]
pub struct Frame {
    content: Vec<CollectionMapping>,
    cause: PathNode,
    parent: Option<usize>,
}

impl Frame {
    fn new(cause: PathNode, parent: Option<usize>, content: Vec<CollectionMapping>) -> Self {
        Frame {
            cause: cause,
            parent: parent,
            content: content,
        }
    }

    pub fn get_content(&self) -> &Vec<CollectionMapping> {
        return &self.content;
    }

    // frame things
    pub fn parent_index(&self) -> Option<usize> {
        return self.parent;
    }

    pub fn get_cause(&self) -> &PathNode {
        return &self.cause;
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

    pub fn get_element(&self, n: i16, node: &GastID) -> Mapping {
        let mut result = Mapping::new();
        let mut count = 0;
        for coll_mapping in self.content.iter() {
            let &CollectionMapping {ref path, ref branch} = coll_mapping;
            
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

    pub fn get_any_element(&self, node: &GastID) -> Mapping {
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

    pub fn get_first_n(&self, n: i16, node: &GastID) -> Vec<Mapping> {
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

    pub fn get_last_n(&self, n: i16, node: &GastID) -> Vec<Mapping> {
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

#[derive(Clone, Debug)]
pub struct Collection {
    frames: Vec<Frame>,
    path: Vec<usize>,
}


impl Collection {
    pub fn new() -> Collection {
        Collection {
            frames: vec![Frame::new(PathNode::Frame(0, None, Box::new(Path::empty())), None, vec!())],
            path: vec!(0),
        }
    }

    pub fn set_content(&mut self, content: Vec<(Path, CollectionBranch)>) {
        if self.frames.len() > 1 {
            panic!("plsno");
        }

        let mut mappings = Vec::new();
        for (path, branch) in content.into_iter() {
            mappings.push(CollectionMapping::new(path, branch));
        }
        self.current_frame_mut(Path::empty()).set_content(mappings);
    }

    // collection things

    pub fn num_frames(&self) -> usize {
        return self.frames.len();
    }

    pub fn current_frame(&self) -> &Frame {
        // last possible frame
        let mut index = self.frames.len() - 1;

        // path describes offset from the last frame
        index -= *self.path.last().unwrap();

        return &self.frames[index];
    }

    pub fn current_frame_mut(&mut self, path: Path) -> &mut Frame {
        let mut count = 0 as usize;
        let mut current_index = 0 as usize;

        for node in path.iter() {
            let old_index = current_index;

            match node {
                &PathNode::Condition(_, b) |
                &PathNode::Loop(_, b) => {
                    count += 2;
                    if b {
                        current_index = count - 1;
                    } else {
                        current_index = count;
                    }
                }
                _ => {
                    count += 1;
                    current_index = count;
                }
            }

            if current_index >= self.frames.len() {
                let current_content = self.frames[old_index].get_content().clone();
                
                match node {
                    &PathNode::Condition(_, b) |
                    &PathNode::Loop(_, b) => {
                        if b {
                            self.path.push(1);
                        } else {
                            self.path.push(0);
                        }
                    }
                    _ => self.path.push(0),
                }

                match node {
                    &PathNode::Condition(l, _) => {
                        let positive = PathNode::Condition(l, true);
                        self.frames.push(Frame::new(positive, Some(old_index.clone()), current_content.clone()));
                        let negative = PathNode::Condition(l, false);
                        self.frames.push(Frame::new(negative, Some(old_index.clone()), current_content.clone()));
                    }
                    &PathNode::Loop(l, _) => {
                        let positive = PathNode::Loop(l, true);
                        self.frames.push(Frame::new(positive, Some(old_index.clone()), current_content.clone()));
                        let negative = PathNode::Loop(l, false);
                        self.frames.push(Frame::new(negative, Some(old_index.clone()), current_content.clone()));
                    }
                    _ => {
                        self.frames.push(Frame::new(node.clone(), Some(old_index.clone()), current_content.clone()));
                    }
                }
            }
        }

        return &mut self.frames[current_index];
    }

    pub fn change_branch(&mut self) {
        let current = self.path.pop().unwrap();
        self.path.push((current + 1) % 2);
    }

    pub fn merge_until(&mut self, cutoff: Option<GastID>) {
        if let Some(cutoff) = cutoff {
            while self.frames.len() > 1 {
                let mut id = 0;

                if let Some(frame) = self.frames.last() {
                    id = frame.cause.get_location().clone();
                } 

                if cutoff >= id {
                    break;
                }
                    
                self.merge_branches();
            }
        } else {
            self.merge_branches();
        }
    }

    pub fn lift_branches(&mut self) {
        self.merge_branches();
    }

    pub fn merge_branches(&mut self) {
        if self.frames.len() < 2 {
            return;
        }

        let mut new_content = Vec::new();
        let first = self.frames.pop().unwrap();
        let second = self.frames.pop().unwrap();

        let cause = first.cause.clone();
        for mapping in first.into_iter() {
            let CollectionMapping {mut path, branch} = mapping;
            path.add_node(cause.clone());
            new_content.push(CollectionMapping::new(path, branch))
        }

        let cause = second.cause.clone();
        for mapping in second.into_iter() {
            let CollectionMapping {mut path, branch} = mapping;
            path.add_node(cause.clone());
            new_content.push(CollectionMapping::new(path, branch))
        }

        let _ = self.path.pop();

        let mut current_index = self.frames.len() - 1;

        current_index -= *self.path.last().unwrap();

        let ref mut current_frame = self.frames[current_index];

        current_frame.set_content(new_content);
    }

    // interface pass-through things

    pub fn size_range(&self) -> Vec<(Path, Option<usize>, Option<usize>)> {
        self.current_frame().size_range()
    }

    pub fn is_reliable(&self) -> Vec<(Path, bool)> {
        self.current_frame().is_reliable()
    }

    pub fn get_element(&self, n: i16, node: &GastID) -> Mapping {
        self.current_frame().get_element(n, node)
    }

    pub fn get_any_element(&self, node: &GastID) -> Mapping {
        self.current_frame().get_any_element(node)
    }

    pub fn get_first_n(&self, n: i16, node: &GastID) -> Vec<Mapping> {
        self.current_frame().get_first_n(n, node)
    }

    pub fn get_last_n(&self, n: i16, node: &GastID) -> Vec<Mapping> {
        self.current_frame().get_last_n(n, node)
    }

    pub fn slice(&self, start: i16, end: i16) -> Vec<(Path, CollectionBranch)> {
        self.current_frame().slice(start, end)
    }

    pub fn insert(&mut self, element: CollectionChunk, path: Path) {
        self.current_frame_mut(path).insert(element)
    }

    pub fn define(&mut self, definition: Vec<CollectionChunk>, path: Path) {
        self.current_frame_mut(path).define(definition)
    }

    pub fn append(&mut self, element: CollectionChunk, path: Path) {
        self.current_frame_mut(path).append(element)
    }

    pub fn prepend(&mut self, element: CollectionChunk, path: Path) {
        self.current_frame_mut(path).prepend(element)
    }
}

fn linearize(n: usize,
                chunks: &[CollectionChunk],
                reverse: bool)
                -> Vec<LinkedList<Mapping>> {
    let chunk;
    let next_chunk;

    if n <= 0 {
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
                let mut intermediate = linearize(n - pls.len(), next_chunk, reverse);

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

                    let mut intermediate = linearize(n - pls.len(), next_chunk, reverse);
                    
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
