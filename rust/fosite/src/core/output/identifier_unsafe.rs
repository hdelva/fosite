use super::Path;
use super::MessageContent;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use super::GastNode;
use super::GastID;

use std::collections::HashMap;

use std::collections::BTreeSet;

use super::IDENTIFIER_UNSAFE;

type Sources = HashMap<GastID, (i16, i16)>;
type Nodes = HashMap<GastID, GastNode>;

pub struct IdentifierUnsafe {
    name: String,
    paths: BTreeSet<Path>,
}

impl IdentifierUnsafe {
    pub fn new(name: String, paths: BTreeSet<Path>) -> Self {
        IdentifierUnsafe {
            name: name,
            paths: paths,
        }
    }
}

impl MessageContent for IdentifierUnsafe {
    fn hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        let mut fingerprint = 0;

        for path in self.paths.iter() {
            for node in path.iter() {
                if node.get_location() > fingerprint {
                    fingerprint = node.get_location()
                }
            }
        }

        IDENTIFIER_UNSAFE.hash(&mut s);
        self.name.hash(&mut s);
        fingerprint.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, source: &Sources, _: &Nodes, node: GastID) {
        self.print_warning_preamble(source, node);
        println!("  New variable {} doesn't always exist",
                 Bold.paint(&self.name));
        println!("  In the following cases:");

        if self.paths.len() == 0 {
            println!("    {}", Red.bold().paint("Always"));
            println!("");
        }

        for (index, path) in self.paths.iter().enumerate() {
            println!("  Case {}",
                Bold.paint(format!("{}", index + 1)));
            self.print_path(source, path, "    ");
            println!("");
        }
    }
}