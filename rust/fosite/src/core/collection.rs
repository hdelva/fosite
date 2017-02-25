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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Representant {
    object: Pointer,
    kind: Pointer,
}

impl Representant {
    pub fn new(object: Pointer, kind: Pointer) -> Representant {
        Representant {
            object: object,
            kind: kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollectionChunk {
    pub minimum: Option<usize>,
    pub maximum: Option<usize>,
    pub representant: Representant,
}

impl CollectionChunk {
    pub fn new(min: Option<usize>, max: Option<usize>, repr: Representant) -> Self {
        CollectionChunk {
            minimum: min,
            maximum: max,
            representant: repr,
        }
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
            min_size = min_size.and_then(|old| chunk.minimum.map(|new| old + new));
            max_size = max_size.and_then(|old| chunk.maximum.map(|new| old + new));
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

    pub fn insert(&mut self, element: Representant, max: Option<usize>) {
        let mut new_content = Vec::new();
        let mut b = false;
        for chunk in self.content.iter() {
            let old_representant = &chunk.representant;
            if element.kind == old_representant.kind {
                let new_chunk = CollectionChunk::new(chunk.minimum, 
                    chunk.maximum.and_then(|old| max.map(|new| old + new)), 
                    old_representant.clone());
                new_content.push(new_chunk);
                b = true;
            } else {
                if !b {
                    let new_chunk = CollectionChunk::new(Some(0), max.clone(), old_representant.clone());
                    new_content.push(new_chunk);
                } 
                new_content.push(chunk.clone());
                b = false;
            }

            self.max_size = self.max_size.and_then(|old| max.map(|new| old + new));
        }

        self.content = new_content;
    }

    pub fn append(&mut self, definition: Representant, min: Option<usize>, max: Option<usize>) {
        self.min_size = self.min_size.and_then(|old| min.map(|new| old + new));
        self.max_size = self.max_size.and_then(|old| max.map(|new| old + new));

        {
            let last = self.content.last_mut();

            if let Some(mut element) = last {
                if element.representant.kind == definition.kind {
                    element.minimum = element.minimum.and_then(|old| min.map(|new| old + new));
                    element.maximum = element.maximum.and_then(|old| max.map(|new| old + new));
                    return;
                }
            }     
        }

        self.content.push(CollectionChunk::new(min, max, definition))
    }

    pub fn prepend(&mut self, definition: Representant, min: Option<usize>, max: Option<usize>) {
        self.min_size = self.min_size.and_then(|old| min.map(|new| old + new));
        self.max_size = self.max_size.and_then(|old| max.map(|new| old + new));

        {
            let first = self.content.first_mut();

            if let Some(mut element) = first {
                if element.representant.kind == definition.kind {
                    element.minimum = element.minimum.and_then(|old| min.map(|new| old + new));
                    element.maximum = element.maximum.and_then(|old| max.map(|new| old + new));
                }
            }
        }

        self.content.insert(0, CollectionChunk::new(min, max, definition));
    }

    fn first_combinations(&self, n: i16) -> Vec<Vec<&CollectionChunk>> {
        let current = Vec::new();
        return flatten(n, current, &self.content, false);
    }

    fn last_combinations(&self, n: i16) -> Vec<Vec<&CollectionChunk>> {
        let current = Vec::new();
        let mut result = flatten(n, current, &self.content, true);
        result.reverse();
        return result;
    }

    pub fn get_element(&self, n: i16) -> Vec<Pointer> {
        let mut result = HashSet::new();
        
        if n < 0 {
            for possibility in self.last_combinations(-n) {
                result.insert(possibility.first().unwrap().representant.object.clone());
            }
        } else {
            for possibility in self.first_combinations(-n) {
                result.insert(possibility.last().unwrap().representant.object.clone());
            }
        }

        return Vec::from_iter(result.into_iter());
    }

    pub fn get_any_element(&self) -> Vec<Pointer> {
        let mut result = HashSet::new();
        for chunk in self.content.iter() {
            result.insert(chunk.representant.object.clone());
        }
        return Vec::from_iter(result.into_iter());
    }

    pub fn get_first_n(&self, n: i16) -> Vec<Vec<Pointer>> {
        let mut result = Vec::new();

        let combinations = self.first_combinations(n);
        //let count = combinations.iter().fold(0, |acc, &x| cmp::max(acc, x.len()));
        for i in 0..n {
            let mut element = Vec::new();
            for combination in combinations.iter() {
                let opt_chunk = combination.get(i as usize);
                if let Some(chunk) = opt_chunk {
                    element.push(chunk.representant.object.clone())
                } else {
                    break;
                }
            }
            result.push(element);
        }

        return result;
    }

    pub fn get_last_n(&self, n: i16) -> Vec<Vec<Pointer>> {
        let mut result = Vec::new();

        let combinations = self.last_combinations(n);
        //let count = combinations.iter().fold(0, |acc, &x| cmp::max(acc, x.len()));
        for i in 0..n {
            let mut element = Vec::new();
            for combination in combinations.iter() {
                let opt_chunk = combination.get(i as usize);
                if let Some(chunk) = opt_chunk {
                    element.push(chunk.representant.object.clone())
                } else {
                    break;
                }
            }
            result.push(element);
        }

        return result;
    }

    pub fn slice(&self, start: i16, end: i16) -> CollectionBranch {
        let mut min_counts: HashMap<CollectionChunk, usize> = HashMap::new();
        let mut max_counts: HashMap<CollectionChunk, usize> = HashMap::new();

        let head = self.first_combinations(start);

        for possibilities in head {
            let mut pls = HashMap::new();
            for chunk in possibilities.iter() {
                *pls.entry(chunk.clone()).or_insert(0) += 1;
            }

            for (chunk, count) in pls.into_iter() {
                match min_counts.entry(chunk.clone()) {
                    Entry::Occupied(mut c) => {
                        *c.get_mut() = cmp::min(c.get().clone(), count);
                    },
                    Entry::Vacant(c) => {
                        c.insert(count);
                    }
                }

                match max_counts.entry(chunk.clone()) {
                    Entry::Occupied(mut c) => {
                        *c.get_mut() = cmp::max(c.get().clone(), count);
                    },
                    Entry::Vacant(c) => {
                        c.insert(count);
                    }
                }
            }
        }

        let tail = self.last_combinations(end);

        for possibility in tail {
            let mut pls = HashMap::new();
            for chunk in possibility.iter() {
                *pls.entry(chunk.clone()).or_insert(0) += 1;
            }

            for (chunk, count) in pls.into_iter() {
                match min_counts.entry(chunk.clone()) {
                    Entry::Occupied(mut c) => {
                        *c.get_mut() = cmp::min(c.get().clone(), count);
                    },
                    Entry::Vacant(c) => {
                        c.insert(count);
                    }
                }

                match max_counts.entry(chunk.clone()) {
                    Entry::Occupied(mut c) => {
                        *c.get_mut() = cmp::max(c.get().clone(), count);
                    },
                    Entry::Vacant(c) => {
                        c.insert(count);
                    }
                }
            }
        }

        let mut new_content = Vec::new();

        for chunk in self.content.iter() {
            let new_min = chunk.minimum.and_then(|old| min_counts.get(chunk).map(|new| old - *new));
            let new_max = chunk.maximum.and_then(|old| max_counts.get(chunk).map(|new| old - *new));

            if new_max.unwrap_or(1) > 0 {
                let new_chunk = CollectionChunk::new(new_min, new_max, chunk.representant.clone());
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

    pub fn insert(&mut self, element: Representant, max: Option<usize>) {
        for mapping in self.content.iter_mut() {
            let &mut CollectionMapping {ref mut branch, ..} = mapping;
            branch.insert(element.clone(), max);
        }
    }

    pub fn define(&mut self, definition: Vec<CollectionChunk>) {
        let new_branch = CollectionBranch::new(definition);
        let mapping = CollectionMapping::new(Path::empty(), new_branch);
        self.content = vec!(mapping);
    }

    pub fn append(&mut self, definition: Representant, min: Option<usize>, max: Option<usize>) {
        for mapping in self.content.iter_mut() {
            let &mut CollectionMapping {ref mut branch, ..} = mapping;
            branch.append(definition.clone(), min, max);
        }
    }

    pub fn prepend(&mut self, definition: Representant, min: Option<usize>, max: Option<usize>) {
        for mapping in self.content.iter_mut() {
            let &mut CollectionMapping {ref mut branch, ..} = mapping;
            branch.prepend(definition.clone(), min, max);
        }
    }

    pub fn get_element(&self, n: i16, node: &GastID) -> Vec<Mapping> {
        let mut result = Vec::new();
        for coll_mapping in self.content.iter() {
            let &CollectionMapping {ref path, ref branch} = coll_mapping;
            let mut count = 0;
            let possibilities = branch.get_element(n);
            let total = possibilities.len().clone() as i16;
            for address in possibilities {
                let mut new_path = path.clone();
                let new_node = PathNode::Element(node.clone(), count, total);
                new_path.add_node(new_node);
                result.push(Mapping::simple(new_path, address.clone()));
                count += 1;
            }
        }
        return result;
    }

    pub fn get_any_element(&self, node: &GastID) -> Vec<Mapping> {
        let mut result = Vec::new();
        for coll_mapping in self.content.iter() {
            let &CollectionMapping {ref path, ref branch} = coll_mapping;
            let mut count = 0;
            let possibilities = branch.get_any_element();
            let total = possibilities.len().clone() as i16;
            for address in possibilities {
                let mut new_path = path.clone();
                let new_node = PathNode::Element(node.clone(), count, total);
                new_path.add_node(new_node);
                result.push(Mapping::simple(new_path, address.clone()));
                count += 1;
            }
        }
        return result;
    }

    pub fn get_first_n(&self, n: i16, node: &GastID) -> Vec<Vec<Mapping>> {
        let mut result = Vec::new();
        for coll_mapping in self.content.iter() {
            let &CollectionMapping {ref path, ref branch} = coll_mapping;
            let mut subresult = Vec::new();
            for collection in branch.get_first_n(n) {
                let total = collection.len().clone() as i16;
                let mut count = 0;
                for address in collection {
                    let mut new_path = path.clone();
                    let new_node = PathNode::Element(node.clone(), count, total);
                    new_path.add_node(new_node);
                    subresult.push(Mapping::simple(new_path, address.clone()));
                    count += 1;
                }
            }
            result.push(subresult)
        }
        return result;
    }

    pub fn get_last_n(&self, n: i16, node: &GastID) -> Vec<Vec<Mapping>> {
        let mut result = Vec::new();
        for coll_mapping in self.content.iter() {
            let &CollectionMapping {ref path, ref branch} = coll_mapping;
            let mut subresult = Vec::new();
            for collection in branch.get_last_n(n) {
                let total = collection.len().clone() as i16;
                let mut count = 0;
                for address in collection {
                    let mut new_path = path.clone();
                    let new_node = PathNode::Element(node.clone(), count, total);
                    new_path.add_node(new_node);
                    subresult.push(Mapping::simple(new_path, address.clone()));
                    count += 1;
                }
            }
            result.push(subresult)
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
            path: vec!(),
        }
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

    pub fn get_element(&self, n: i16, node: &GastID) -> Vec<Mapping> {
        self.current_frame().get_element(n, node)
    }

    pub fn get_any_element(&self, node: &GastID) -> Vec<Mapping> {
        self.current_frame().get_any_element(node)
    }

    pub fn get_first_n(&self, n: i16, node: &GastID) -> Vec<Vec<Mapping>> {
        self.current_frame().get_first_n(n, node)
    }

    pub fn get_last_n(&self, n: i16, node: &GastID) -> Vec<Vec<Mapping>> {
        self.current_frame().get_last_n(n, node)
    }

    pub fn slice(&self, start: i16, end: i16) -> Vec<(Path, CollectionBranch)> {
        self.current_frame().slice(start, end)
    }

    pub fn insert(&mut self, element: Representant, max: Option<usize>, path: Path) {
        self.current_frame_mut(path).insert(element, max)
    }

    pub fn define(&mut self, definition: Vec<CollectionChunk>, path: Path) {
        self.current_frame_mut(path).define(definition)
    }

    pub fn append(&mut self, definition: Representant, min: Option<usize>, max: Option<usize>, path: Path) {
        self.current_frame_mut(path).append(definition, min, max)
    }

    pub fn prepend(&mut self, definition: Representant, min: Option<usize>, max: Option<usize>, path: Path) {
        self.current_frame_mut(path).prepend(definition, min, max)
    }
}

fn flatten<'a, 'b>(n: i16,
                mut current: Vec<&'a CollectionChunk>,
                chunks: &'a [CollectionChunk],
                reverse: bool)
                -> Vec<Vec<&'a CollectionChunk>> {
    let chunk;
    let next_chunk;

    if current.len() == n as usize {
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
        Some(element) => {
            let mut result = Vec::new();

            for _ in 0..element.minimum.unwrap_or(n as usize) {
                current.push(&element);
                if current.len() >= n as usize {
                    result.push(current);
                    return result;
                }
            }

            // minimum has been reached, we can now move to the next chunk
            let mut partial = flatten(n, current.clone(), next_chunk, reverse);
            result.append(&mut partial);

            // or rather than going to the next chunk,
            // add more until we reach the maximum number
            for _ in element.minimum.unwrap_or(0)..element.maximum.unwrap_or(n as usize) {
                current.push(&element);

                if current.len() >= n as usize {
                    result.push(current);
                    return result;
                } else {
                    let mut partial = flatten(n, current.clone(), next_chunk, reverse);
                    result.append(&mut partial);
                }

            }

            return result;
        }
        _ => panic!("not enough elements to unpack"),
    }
}
