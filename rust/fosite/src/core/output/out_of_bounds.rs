use super::Path;
use super::MessageContent;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use std::collections::HashMap;
use super::GastID;
use super::GastNode;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use super::OUT_OF_BOUNDS;

type Sources = HashMap<GastID, (i16, i16)>;
type Nodes = HashMap<GastID, GastNode>;

pub struct OutOfBounds {
    target: String,
    cases: Vec<(Path, i16)>,
}

impl OutOfBounds {
    pub fn new(target: String, cases: Vec<(Path, i16)>) -> Self {
        OutOfBounds {
            target: target,
            cases: cases,
        }
    }
}


impl MessageContent for OutOfBounds {
    fn hash(&self) -> u64 {
        let mut s = DefaultHasher::new();

        let mut fingerprint = Path::empty();

        for &(ref path, _) in self.cases.iter() {
            if path > &fingerprint {
                fingerprint = path.clone();
            }
        }

        OUT_OF_BOUNDS.hash(&mut s);
        self.target.hash(&mut s);
        fingerprint.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, _: &Nodes, node: GastID) {
        self.print_warning_preamble(sources, node);
        println!("  Index might be out of bounds");
        println!("  {} does not always have enough elements",
                 Bold.paint(&self.target));
        println!("  In the following cases:");


        for (index, &(ref path, max)) in self.cases.iter().enumerate() {
            println!("  Case {}",
                    Bold.paint(format!("{}", index + 1)));
            println!("    {} has {} elements at most in the following case", self.target, max);
            if path.len() > 0 {
                self.print_path(sources, &path, "    ");
            } else {
                println!("    {}", Red.bold().paint("Always"));
            }
            
            println!("");
        }
    }
}