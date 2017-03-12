
use super::Path;
use super::MessageContent;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

type Sources = HashMap<GastID, (i16, i16)>;
type Nodes = HashMap<GastID, GastNode>;

use std::collections::HashMap;
use super::GastID;
use super::GastNode;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;

use super::IDENTIFIER_INVALID;

pub struct IdentifierInvalid {
    name: String,
    paths: BTreeSet<Path>,
}

impl IdentifierInvalid {
    pub fn new(name: String, paths: BTreeSet<Path>) -> Self {
        IdentifierInvalid {
            name: name,
            paths: paths,
        }
    }
}

impl MessageContent for IdentifierInvalid {
    fn hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        IDENTIFIER_INVALID.hash(&mut s);
        self.name.hash(&mut s);
        self.paths.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, _: &Nodes, node: GastID) {
        self.print_error_preamble(sources, node);
        println!("  {} does not exist",
                 Bold.paint(&self.name));
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