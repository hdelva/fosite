#![feature(box_syntax)]
#![allow(dead_code)]

use std::collections::HashSet;

trait KnowledgeBase {
    fn types_with_attribute(&self, name: &str) -> HashSet<i32>;
    fn iterable_types(&self) -> HashSet<i32>;
    fn indexable_types(&self) -> HashSet<i32>;
}

// Objects

trait Object {
    fn get_types(&self) -> &[String];

    fn set_strategy(&self, strategy: Box<ObjectStrategy>);
    fn get_strategy(&self) -> ObjectStrategy;

    fn assign_attribute(&self, name: &str);
    fn reference_attribute(&self, name: &str);
    fn iterate(&self);
    fn index(&self);

    //todo some way to return function definitions
}

trait ObjectStrategy {
    fn assign_attribute(&mut self, kb: &KnowledgeBase, parent: &Object, name: &str);
    fn reference_attribute(&mut self, kb: &KnowledgeBase, parent: &Object, name: &str);
    fn iterate(&mut self, kb: &KnowledgeBase, parent: &Object, name: &str);
    fn index(&mut self, kb: &KnowledgeBase, parent: &Object, name: &str);
}

struct BottomObjectStrategy {

}

impl ObjectStrategy for BottomObjectStrategy {
    fn assign_attribute(&mut self, kb: &KnowledgeBase, parent: &Object, name: &str) {
        //
    }

    fn reference_attribute(&mut self, kb: &KnowledgeBase, parent: &Object, name: &str) {
        //
    }

    fn iterate(&mut self, kb: &KnowledgeBase, parent: &Object, name: &str) {
        //
    }

    fn index(&mut self, kb: &KnowledgeBase, parent: &Object, name: &str) {
        //
    }
}

struct CustomObjectStrategy {
    attributes: HashSet<String>,
    types: HashSet<i32>,
}

impl ObjectStrategy for CustomObjectStrategy {
    fn assign_attribute(&mut self, kb: &KnowledgeBase, parent: &Object, name: &str) {
        self.attributes.insert(String::from(name));
    }

    fn reference_attribute(&mut self, kb: &KnowledgeBase, parent: &Object, name: &str) {
        if !self.attributes.contains(name) {
            parent.set_strategy(box BottomObjectStrategy {});
        }
    }

    fn iterate(&mut self, kb: &KnowledgeBase, parent: &Object, name: &str) {
        let iterable_types = kb.iterable_types();

        let possible_types: HashSet<_> = iterable_types.intersection(&self.types).cloned().collect();

        if possible_types.len() > 0 {
            self.types = possible_types;
        } else {
            parent.set_strategy(box BottomObjectStrategy {});
        }
    }

    fn index(&mut self, kb: &KnowledgeBase, parent: &Object, name: &str) {
        let indexable_types = kb.indexable_types();

        let possible_types: HashSet<_> = indexable_types.intersection(&self.types).cloned().collect();

        if possible_types.len() > 0 {
            self.types = possible_types;
        } else {
            parent.set_strategy(box BottomObjectStrategy {});
        }
    }
}


fn main() {
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);

    println!("Hello, world!");
}
