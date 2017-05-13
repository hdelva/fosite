
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
use super::PathID;
use super::NodeType;


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
    fn hash(&self, source: &PathID) -> u64 {
        let mut s = DefaultHasher::new();
        IDENTIFIER_INVALID.hash(&mut s);
        self.name.hash(&mut s);
        self.paths.hash(&mut s);
        source.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, nodes: &Nodes, node: PathID) {
        let source_node = node.last().unwrap().clone();
        let node_type = nodes.get(&source_node).unwrap();

        match &node_type.kind {
            &NodeType::Identifier {..} => (),
            _ => return
        }

        self.print_error_preamble(sources, node);
        println!("  {} does not exist",
                 Bold.paint(&self.name));
        println!("  In the following cases:");

        let relevant_paths = self.reduce_paths(sources, &self.paths);

        if relevant_paths.len() == 0 {
            println!("    {}", Red.bold().paint("Always"));
            println!("");
        }

        for (index, path) in relevant_paths.iter().enumerate() {
            println!("  Case {}",
                Bold.paint(format!("{}", index + 1)));
            self.print_path(sources, path, "    ");
            println!("");
        }
    }
}

