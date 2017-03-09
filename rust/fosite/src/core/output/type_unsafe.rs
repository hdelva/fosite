use super::Path;
use super::MessageContent;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use super::PathNode;

use std::hash::{Hash, Hasher, SipHasher};
use super::GastID;
use super::GastNode;

type Sources = HashMap<GastID, (i16, i16)>;
type Nodes = HashMap<GastID, GastNode>;

pub struct TypeUnsafe {
    name: String,
    types: BTreeMap<String, Vec<Path>>,
}

impl TypeUnsafe {
    pub fn new(name: String, types: BTreeMap<String, Vec<Path>>) -> Self {
        TypeUnsafe {
            name: name,
            types: types,
        }
    }
}

impl MessageContent for TypeUnsafe {
    fn hash(&self) -> u64 {
        let mut s = SipHasher::new();
        let mut set = BTreeSet::new();

        for (index, (_, paths)) in self.types.iter().enumerate() {
            for path in paths.iter() {
                match path.iter().next_back() {
                    Some(thing) => set.insert((index as i16, thing.clone())),
                    _ => set.insert((0, PathNode::Condition(0, true))),
                };
            }
        }

        self.name.hash(&mut s);
        set.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, nodes: &Nodes, node: GastID) {
        self.print_warning_preamble(sources, node);
        println!("  Not all code paths give {} the same type",
                 Bold.paint(self.name.clone()));
        println!("  In the following cases:");

        for (t_index, (t, paths)) in self.types.iter().enumerate() {
            println!("  Type {}: {}",
                t_index,
                Bold.paint(t.clone()));

            for path in paths.iter() {
                self.print_path(sources, path, "    ");
                println!("");
            }
        }
    }
}