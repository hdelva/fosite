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


use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use super::FOR_LOOP_CHANGE;

pub struct ForLoopChange {
    paths: Vec<Path>,
}

impl ForLoopChange {
    pub fn new(paths: Vec<Path>) -> Self {
        ForLoopChange {
            paths: paths,
        }
    }
}

impl MessageContent for ForLoopChange {
    fn hash(&self, _: &PathID) -> u64 {
        let mut s = DefaultHasher::new();
        FOR_LOOP_CHANGE.hash(&mut s);
        self.paths.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, _: &Nodes, node: PathID) {
        self.print_warning_preamble(sources, node);
        println!("  Some code paths change the collection that's being iterated over");
        println!("  This can have unexpected consequences");
        println!("  In the following cases:");

        if self.paths.first().unwrap().is_empty() {
            println!("    {}", Red.bold().paint("Always"));
            println!("");
        } else {
            for (index, path) in self.paths.iter().enumerate() {
                println!("  Case {}",
                    Bold.paint(format!("{}", index + 1)));
                self.print_path(sources, path, "    ");
                println!("");
            }
        }
    }
}