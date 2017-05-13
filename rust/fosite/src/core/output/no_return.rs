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

use std::collections::BTreeSet;

use super::NO_RETURN;

pub struct NoReturn {
    paths: BTreeSet<Path>,
}

impl NoReturn {
    pub fn new(paths: BTreeSet<Path>) -> Self {
        NoReturn {
            paths: paths,
        }
    }
}

impl MessageContent for NoReturn {
    fn hash(&self, _: &PathID) -> u64 {
        let mut s = DefaultHasher::new();
        NO_RETURN.hash(&mut s);
        self.paths.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, _: &Nodes, node: PathID) {
        self.print_warning_preamble(sources, node);
        println!("  Not all code paths have returned a value");
        println!("  Python will pretend a None value was returned");
        println!("  In the following cases:");

        if self.paths.len() == 0 || self.paths.iter().next().unwrap().len() == 0{
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