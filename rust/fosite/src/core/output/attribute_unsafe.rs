use super::Path;
use super::MessageContent;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use super::GastID;
use super::GastNode;

use std::hash::{Hash, Hasher, SipHasher};
use std::collections::HashMap;
use std::collections::BTreeSet;

type Sources = HashMap<GastID, (i16, i16)>;
type Nodes = HashMap<GastID, GastNode>;


pub struct AttributeUnsafe {
    parent: String,
    attribute: String,
    paths: BTreeSet<Path>,
}

impl AttributeUnsafe {
    pub fn new(parent: String, attribute: String, paths: BTreeSet<Path>) -> Self {
        AttributeUnsafe {
            parent: parent,
            attribute: attribute,
            paths: paths,
        }
    }
}

impl MessageContent for AttributeUnsafe {
    fn hash(&self) -> u64 {
        let mut s = SipHasher::new();

        let mut fingerprint = 0;

        for path in self.paths.iter() {
            for node in path.iter() {
                if node.get_location() > fingerprint {
                    fingerprint = node.get_location()
                }
            }
        }

        self.parent.hash(&mut s);
        self.attribute.hash(&mut s);
        fingerprint.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, nodes: &Nodes, node: GastID) {
        self.print_warning_preamble(sources, node);
        println!("  Object {} does not always have an attribute {}",
                 Bold.paint(&self.parent),
                 Bold.paint(&self.attribute));
        println!("  In the following cases:");

        if self.paths.len() == 0 {
            println!("    {}", Red.bold().paint("Always"));
            println!("");
        }

        for (index, path) in self.paths.iter().enumerate() {
            println!("  Case {}",
                Bold.paint(format!("{}", index + 1)));
            self.print_path(sources, path, "    ");
            println!("");
        }
    }
}