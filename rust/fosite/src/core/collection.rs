use super::Pointer;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::Entry;
use std::cmp;
use super::Mapping;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    minimum: i16,
    maximum: i16,
    representant: Representant,
}

impl Chunk {
    fn new(min: i16, max: i16, repr: Representant) -> Chunk {
        Chunk {
            minimum: min,
            maximum: max,
            representant: repr,
        }
    }
}

#[derive(Clone, Debug)]
struct Branch {
    content: Vec<Chunk>,
}

impl Branch {
    pub fn new(content: Vec<Chunk>) -> Branch {
        Branch { 
            content: content,
        }
    }

    pub fn insert(&mut self, element: Representant) {
        let mut new_content = Vec::new();
        let mut b = false;
        for chunk in self.content {
            let old_representant = &chunk.representant;
            if element.kind == old_representant.kind {
                let new_chunk = Chunk::new(chunk.min, chunk.max + 1, old_representant.clone());
                new_content.push(new_chunk);
                b = true;
            } else {
                if !b {
                    let new_chunk = Chunk::new(0, 1, old_representant.clone());
                    new_content.push(new_chunk);
                } 
                new_content.push(chunk.clone())
                b = false;
            }
        }
        self.content = new_content;
    }

    pub fn define(&mut self, definition: Vec<Representant>) {
        let mut content = vec!();

        for object in definition {
            content.push(Chunk::new(1, 1, object))
        }

        self.content = content;
    }

    pub fn append(&mut self, definition: Representant, min: i16, max: i16) {
        {
            let last = self.content.last_mut();

            if let Some(mut element) = last {
                if element.representant.kind == definition.kind {
                    element.minimum += min;
                    element.maximum += max;
                    return;
                }
            }     
        }

        self.content.push(Chunk::new(min, max, definition))
    }

    pub fn prepend(&mut self, definition: Representant, min: i16, max: i16) {
        {
            let first = self.content.first_mut();

            if let Some(mut element) = first {
                if element.representant.kind == definition.kind {
                    element.minimum += min;
                    element.maximum += max;
                }
            }
        }

        content.insert(0, Chunk::new(min, max, definition));
    }

    fn first_combinations(&self, n: i16) -> Vec<Vec<&Chunk>> {
        let current = Vec::new();
        return flatten(n, current, self.content, false);
    }

    fn last_combinations(&self, n: i16) -> Vec<Vec<&Chunk>> {
        let current = Vec::new();
        let mut result = flatten(n, current, self.content, true);
        result.reverse();
        return result;
    }

    pub fn iterate(&self) -> Vec<Pointer> {
        let mut result = Vec::new();
        for chunk in self.content {
            result.push(chunk.object.clone());
        }
        return result;
    }

    pub fn get_first_n(&self, n: i16) -> Vec<Vec<Pointer>> {
        let mut result = Vec::new();

        let combinations = self.first_combinations(n);
        let count = combinations.iter().fold(0, |acc, &x| cmp:max(acc, x));
        for i in 0..count {
            let element = Vec::new();
            for combination in combinations {
                let opt_chunk = combination.get(i);
                if let Some(chunk) = opt_chunk {
                    element.push(chunk.representant.object().clone())
                } else {
                    break;
                }
            }
            result.push(element);
        }
    }

    pub fn get_last_n(&self, n: 16) -> Vec<Vec<Pointer>> {
        let mut result = Vec::new();

        let combinations = self.last_combinations(n);
        let count = combinations.iter().fold(0, |acc, &x| cmp:max(acc, x));
        for i in 0..count {
            let element = Vec::new();
            for combination in combinations {
                let opt_chunk = combination.get(i);
                if let Some(chunk) = opt_chunk {
                    element.push(chunk.representant.object().clone())
                } else {
                    break;
                }
            }
            result.push(element);
        }
    }

    pub fn slice(&self, start: i16, end: i16) -> Branch {
        let mut min_counts = HashMap::new();
        let mut max_counts = HashMap::new();

        let head = self.get_first_n(start);

        for possibility in head {
            let pls = HashMap::new();
            for chunk in possibility.iter() {
                *pls.entry(chunk.clone()).or_insert(0) += 1;
            }

            for (chunk, count) in pls.iter() {
                match min_counts.entry(chunk.clone()) {
                    Occupied(mut c) => {
                        *c.get_mut() = cmp::min(*c.get(), *count);
                    },
                    Vacant(mut c) => {
                        c.insert(*count);
                    }
                }

                match max_counts.entry(chunk.clone()) {
                    Occupied(mut c) => {
                        *c.get_mut() = cmp::max(*c.get(), *count);
                    },
                    Vacant(mut c) => {
                        c.insert(*count);
                    }
                }
            }
        }

        let tail = self.get_last_n(end);

        for possibility in tail {
            let pls = HashMap::new();
            for chunk in possibility.iter() {
                *pls.entry(chunk.clone()).or_insert(0) += 1;
            }

            for (chunk, count) in pls.iter() {
                match min_counts.entry(chunk.clone()) {
                    Occupied(mut c) => {
                        *c.get_mut() = cmp::min(*c.get(), *count);
                    },
                    Vacant(mut c) => {
                        c.insert(*count);
                    }
                }

                match max_counts.entry(chunk.clone()) {
                    Occupied(mut c) => {
                        *c.get_mut() = cmp::max(*c.get(), *count);
                    },
                    Vacant(mut c) => {
                        c.insert(*count);
                    }
                }
            }
        }

        let mut new_content = Vec::new();

        for chunk in self.content.iter() {
            let new_min = chunk.min - min_counts.get(chunk).unwrap_or(0);
            let new_max = chunk.max - max_counts.get(chunk).unwrap_or(0);

            if new_max > 0 {
                let new_chunk = Chunk::new(new_min, new_max, chunk.representant.clone());
                new_content.push(new_chunk);
            }
        }

        return Branch::new(new_content);
    }
}

pub struct Thing {
    pub branch: Branch,
    pub path: Path,
}

impl Thing {
    pub impl new(branch: Branch, path: Path) -> Self {
        Thing {
            branch: branch,
            path: path,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Collection {
    frames: Vec<Frame>,
    path: Vec<usize>,
}


impl Collection {
    pub fn empty() -> Collection {
        Collection {
            frames: vec!(Frame::new()),
            path: vec!(),
        }
    }

    pub fn current_branch(&self, name: &String) -> &Branch {
        // last possible frame
        let mut index = self.frames.len() - 1;

        // path describes offset from the last frame
        index -= *self.path.last().unwrap();

        return self.frames[index];
    }

    pub fn current_branch_mut(&mut self, name: &String) -> &mut Branch {
        // last possible frame
        let mut index = self.frames.len() - 1;

        // path describes offset from the last frame
        index -= *self.path.last().unwrap();

        return self.frames.get_mut(current_index);
    }

    pub fn change_branches(&mut self) {
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

    pub fn merge_branches(&mut self) {
        if self.frames.len() == 1 {
            return;
        }

        let mut new_content = HashMap::new();
    }

    pub fn define(&mut self, definition: Vec<Representant>) {
        
    }

    pub fn append(&mut self, definition: Representant, min: i16, max: i16) {

    }

    pub fn prepend(&mut self, definition: Representant, min: i16, max: i16) {

    }

    pub fn get_first_n(&self, n: i16) -> Vec<Vec<&Chunk>> {
        
    }

    pub fn get_last_n(&self, n: i16) -> Vec<Vec<&Chunk>> {

    }

    /*
    // todo
    fn get_exact_n(&self, n: i16) {

        // return intersect
    }
    */

    pub fn slice(&self, start: i16, end: i16) -> Vec<Vec<&Chunk>> {

    }
}

fn flatten<'a, 'b>(n: i16,
                mut current: Vec<&'a Chunk>,
                chunks: &'a [Chunk],
                reverse: bool)
                -> Vec<Vec<&'a Chunk>> {
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

            for _ in 0..element.minimum {
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
            for _ in element.minimum..element.maximum {
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
