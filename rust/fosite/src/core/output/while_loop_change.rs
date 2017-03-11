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

pub struct WhileLoopChange {
    paths: Vec<Path>,
}

impl WhileLoopChange {
    pub fn new(paths: Vec<Path>) -> Self {
        WhileLoopChange {
            paths: paths,
        }
    }
}

impl MessageContent for WhileLoopChange {
    fn hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.paths.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, _: &Nodes, node: GastID) {
        self.print_warning_preamble(sources, node);
        println!("  Not all code paths update the loop condition");
        println!("  There's a risk of endless loops");
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