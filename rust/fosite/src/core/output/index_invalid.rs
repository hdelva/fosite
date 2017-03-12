use super::MessageContent;

use term_painter::ToStyle;
use term_painter::Attr::*;

use std::collections::HashMap;
use super::GastID;
use super::GastNode;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use std::collections::BTreeMap;

use super::Path;

use super::INDEX_INVALID;

type Sources = HashMap<GastID, (i16, i16)>;
type Nodes = HashMap<GastID, GastNode>;

pub struct IndexInvalid {
    target: String,
    types: BTreeMap<String, Vec<Path>>,
}

impl IndexInvalid {
    pub fn new(target: String, types: BTreeMap<String, Vec<Path>>) -> Self {
        IndexInvalid {
            target: target,
            types: types,
        }
    }
}

impl MessageContent for IndexInvalid {
    fn hash(&self) -> u64 {
        let mut s = DefaultHasher::new();

        INDEX_INVALID.hash(&mut s);
        self.target.hash(&mut s);
        self.types.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, _: &Nodes, node: GastID) {
        self.print_error_preamble(sources, node);
        println!("  {} does not support indexing", 
            Bold.paint(&self.target));
        println!("  It has an incompatible type in the following cases:");

        for (t_index, (t, paths)) in self.types.iter().enumerate() {
            println!("  Type {}: {}",
                t_index + 1,
                Bold.paint(t.clone()));

            for path in paths.iter() {
                self.print_path(sources, path, "    ");
                println!("");
            }
        }
    }
}