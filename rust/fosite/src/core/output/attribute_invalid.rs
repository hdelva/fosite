use super::Path;
use super::MessageContent;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use std::collections::HashMap;
use super::GastID;
use super::GastNode;

use std::hash::{Hash, Hasher, SipHasher};

use std::collections::BTreeSet;

type Sources = HashMap<GastID, (i16, i16)>;
type Nodes = HashMap<GastID, GastNode>;

pub struct AttributeInvalid {
    parent: String,
    attribute: String,
    paths: BTreeSet<Path>,
}

impl AttributeInvalid {
    pub fn new(parent: String, attribute: String, paths: BTreeSet<Path>) -> Self {
        AttributeInvalid {
            parent: parent,
            attribute: attribute,
            paths: paths,
        }
    }
}

impl MessageContent for AttributeInvalid {
    fn hash(&self) -> u64 {
        let mut s = SipHasher::new();
        self.parent.hash(&mut s);
        self.attribute.hash(&mut s);
        self.paths.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, nodes: &Nodes, node: GastID) {
        self.print_error_preamble(sources, node);
        println!("  Object {} does not have an attribute {}",
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