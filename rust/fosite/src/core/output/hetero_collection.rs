use super::MessageContent;

use term_painter::ToStyle;
use term_painter::Attr::*;

use std::collections::HashMap;
use super::GastID;
use super::GastNode;
use super::PathID;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use super::HETERO_COLLECTION;

type Sources = HashMap<GastID, (i16, i16)>;
type Nodes = HashMap<GastID, GastNode>;

pub struct HeteroCollection {
    target: String,
    old_type: String,
    new_type: String,
}

impl HeteroCollection {
    pub fn new(target: String, old: String, new: String) -> Self {
        HeteroCollection {
            target: target,
            old_type: old,
            new_type: new,
        }
    }
}

impl MessageContent for HeteroCollection {
    fn hash(&self, _: &PathID) -> u64 {
        let mut s = DefaultHasher::new();

        HETERO_COLLECTION.hash(&mut s);
        self.target.hash(&mut s);
        self.old_type.hash(&mut s);
        self.new_type.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, _: &Nodes, node: PathID) {
        self.print_warning_preamble(sources, node);
        println!("  Adding an element of a new type to a collection");
        println!("  {} had type {} and became {}",
            Bold.paint(&self.target), 
            Bold.paint(&self.old_type),
            Bold.paint(&self.new_type));
        println!("  This can make working with this collection more difficult \n");
    }
}