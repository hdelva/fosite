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

use super::WHILE_LOOP_CHANGE;

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
    fn hash(&self, _: &PathID) -> u64 {
        let mut s = DefaultHasher::new();
        WHILE_LOOP_CHANGE.hash(&mut s);
        self.paths.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, _: &Nodes, node: PathID) {
        self.print_warning_preamble(sources, node);
        println!("  Not all code paths update the loop condition");
        println!("  There's a risk of endless loops");
        println!("  In the following cases:");

        if self.paths.first().unwrap().len() == 0 {
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