use super::Path;
use super::MessageContent;

use term_painter::ToStyle;
use term_painter::Attr::*;

use std::collections::HashMap;
use super::GastID;
use super::GastNode;
use super::PathID;


type Sources = HashMap<GastID, (i16, i16)>;
type Nodes = HashMap<GastID, GastNode>;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use super::ARGUMENT_INVALID;

pub struct ArgInvalid {
    index: &'static str,
    permitted: Vec<&'static str>,
    actual: Vec<(Path, String)>,
}

impl ArgInvalid {
    pub fn new(index: &'static str, permitted: Vec<&'static str>, actual: Vec<(Path, String)>) -> Self {
        ArgInvalid {
            index: index,
            permitted: permitted,
            actual: actual,
        }
    } 
}

impl MessageContent for ArgInvalid {
    fn hash(&self, _: &PathID) -> u64 {
        let mut s = DefaultHasher::new();
        ARGUMENT_INVALID.hash(&mut s);
        self.index.hash(&mut s);
        self.permitted.hash(&mut s);
        self.actual.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, _: &Nodes, node: PathID) {
        self.print_error_preamble(sources, node);
        println!("  Invalid argument type");
        println!("  The {} Argument should have one of the following types:\n      {:?}",
            &self.index,
            Bold.paint(&self.permitted));
        println!("  It has an invalid type in the following cases:");

        for &(ref path, ref t) in &self.actual {
            println!("    Type {} in the following case:",
                Bold.paint(&t));

            self.print_path(sources, path, "      ");
            println!("");
        }
    }
}