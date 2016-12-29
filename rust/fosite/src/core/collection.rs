use super::Pointer;

use std::hash::{Hash, Hasher};

#[derive(Clone)]
struct Branch {
    content: Vec<Chunk>,
}

impl Branch {
    fn new(original: Vec<Chunk>) -> Branch {
        Branch { content: original.clone() }
    }
}

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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Clone)]
pub struct Collection {
    branches: Vec<Branch>,
    id: i32,
}

impl Hash for Collection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.id)
    }
}

impl PartialEq for Collection {
    fn eq(&self, other: &Collection) -> bool {
        self.id == other.id
    }
}

impl Eq for Collection {}


impl Collection {
    pub fn empty() -> Collection {
        Collection {
            branches: vec![Branch::new(vec![])],
            id: 1,
        }
    }

    pub fn swap(&mut self) {
        let last = self.branches.pop().unwrap();
        let first = self.branches.pop().unwrap();
        self.branches.push(last);
        self.branches.push(first);
    }

    pub fn merge(&mut self) {
        let last = self.branches.pop().unwrap().content;
        let first = self.branches.pop().unwrap().content;
        let mut new_content = vec![];

        let mut last_id = 0;
        let mut first_id = 0;

        while last_id < last.len() && first_id < first.len() {
            let current_last = last[last_id].clone();
            let current_first = first[first_id].clone();

            if current_last == current_first {
                new_content.push(current_last);
                last_id += 1;
                first_id += 1;
            } else if last_id < first_id {
                new_content.push(current_last);
                last_id += 1;
            } else if first_id < last_id {
                new_content.push(current_first);
                first_id += 1;
            }
        }
    }

    pub fn get_content(&self) -> &Vec<Chunk> {
        match self.branches.last() {
            Some(branch) => &branch.content,
            _ => panic!("collection object has no content anymore"),
        }
    }

    pub fn get_content_mut(&mut self) -> &mut Vec<Chunk> {
        match self.branches.last_mut() {
            Some(branch) => &mut branch.content,
            _ => panic!("collection object has no content anymore"),
        }
    }

    pub fn define(&mut self, definition: Vec<Representant>) {
        let mut content = self.get_content_mut();

        for object in definition {
            content.push(Chunk::new(1, 1, object))
        }
    }

    pub fn append(&mut self, definition: Representant, min: i16, max: i16) {
        let mut content = self.get_content_mut();
        {
            let last = content.last_mut();

            match last {
                Some(mut element) => {
                    if element.representant.kind == definition.kind {
                        element.minimum += min;
                        element.maximum += max;
                        return;
                    }
                }
                _ => (),
            }
        }

        content.push(Chunk::new(min, max, definition))

    }

    pub fn prepend(&mut self, definition: Representant, min: i16, max: i16) {
        let mut content = self.get_content_mut();
        {
            let first = content.first_mut();

            match first {
                Some(mut element) => {
                    if element.representant.kind == definition.kind {
                        element.minimum += min;
                        element.maximum += max;
                    }
                }
                _ => (),
            }
        }

        content.insert(0, Chunk::new(min, max, definition));

    }

    pub fn get_first_n(&self, n: i16) -> Vec<Vec<&Chunk>> {
        let current = Vec::new();
        let content = self.get_content();
        return self.fuck(n, current, content, false);
    }

    pub fn get_last_n(&self, n: i16) -> Vec<Vec<&Chunk>> {
        let current = Vec::new();
        let content = self.get_content();
        let mut result = self.fuck(n, current, content, true);
        result.reverse();
        return result;
    }

    // todo
    fn get_exact_n(&self, n: i16) {

        // return intersect
    }

    pub fn slice(&self, start: i16, end: i16) -> Vec<Vec<&Chunk>> {
        let head = self.get_first_n(start);
        let mut start_positions = Vec::new();

        if head.len() == 0 {
            start_positions.push(0);
        }

        for possibility in head {
            let last_head = possibility.last().unwrap();

            let mut count = 0;
            for chunk in possibility.iter().rev() {
                if last_head == chunk {
                    count += 1
                } else {
                    break;
                }
            }

            let mut cloned = possibility.clone();
            cloned.dedup();

            if count < last_head.maximum {
                start_positions.push(cloned.len() - 1);
            } else {
                start_positions.push(cloned.len());
            }
        }

        let tail = self.get_last_n(end);
        let mut end_positions = Vec::new();

        let content = self.get_content();

        if tail.len() == 0 {
            end_positions.push(content.len())
        }

        for possibility in tail {
            let first_tail = possibility.first().unwrap();

            let mut count = 0;
            for chunk in possibility.iter() {
                if first_tail == chunk {
                    count += 1
                } else {
                    break;
                }
            }

            let mut cloned = possibility.clone();
            cloned.dedup();

            if count < first_tail.maximum {
                end_positions.push(content.len() - cloned.len() + 1);
            } else {
                end_positions.push(content.len() - cloned.len());
            }
        }

        let mut result = Vec::new();

        for start in start_positions {
            for end in &end_positions {
                let mut temp = Vec::new();
                for i in start..*end {
                    temp.push(&content[i]);
                }
                result.push(temp);
            }
        }

        return result;
    }

    fn fuck<'a, 'b>(&'a self,
                    n: i16,
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
                let mut partial = self.fuck(n, current.clone(), next_chunk, reverse);
                result.append(&mut partial);

                // or rather than going to the next chunk,
                // add more until we reach the maximum number
                for _ in element.minimum..element.maximum {
                    current.push(&element);

                    if current.len() >= n as usize {
                        result.push(current);
                        return result;
                    } else {
                        let mut partial = self.fuck(n, current.clone(), next_chunk, reverse);
                        result.append(&mut partial);
                    }

                }

                return result;
            }
            _ => panic!("not enough elements to unpack"),
        }


    }
}