use super::Path;
use super::MessageContent;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use std::collections::HashMap;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use super::GastID;
use super::GastNode;
use super::PathID;


type Sources = HashMap<GastID, (i16, i16)>;
type Nodes = HashMap<GastID, GastNode>;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use super::BINOP_INVALID;

pub struct BinOpInvalid {
    operator: String,
    combinations: BTreeMap<(String, String), (BTreeSet<Path>, BTreeSet<Path>)>,
}

impl BinOpInvalid {
    pub fn new(op: String, comb: BTreeMap<(String, String), (BTreeSet<Path>, BTreeSet<Path>)>) -> Self {
        BinOpInvalid {
            operator: op,
            combinations: comb,
        }
    } 
}

impl MessageContent for BinOpInvalid {
    fn hash(&self, _: &PathID) -> u64 {
        let mut s = DefaultHasher::new();
        BINOP_INVALID.hash(&mut s);
        self.operator.hash(&mut s);
        self.combinations.hash(&mut s);
        s.finish()
    }

    fn print_message(&self, sources: &Sources, _: &Nodes, node: PathID) {
        self.print_error_preamble(sources, node);
        println!("  Incompatible types for operation {}",
            Bold.paint(&self.operator));
        println!("  The following combinations exist:");

        for (index, (types, paths)) in self.combinations.iter().enumerate() {
            let &(ref left_type, ref right_type) = types;
            let &(ref left_paths, ref right_paths) = paths;

            println!("  Combination {}: {}",
                     index,
                     Bold.paint(format!("{} {} {}", left_type, self.operator, right_type)));
            
            // left side
            println!("    Left side has type {}", left_type);
            println!("    In the following cases:");

            if left_paths.iter().next().unwrap().len() == 0 {
                println!("      {}", Red.bold().paint("Always"));
                println!("");
            }

            for path in left_paths.iter() {
            //for path in self.reduce_paths(sources, &left_paths) {
                self.print_path(sources, &path, "      ");
                println!("");
            }

            // right side
            println!("    Right side has type {}", right_type);
            println!("    In the following cases:");

            if right_paths.iter().next().unwrap().len() == 0 {
                println!("      {}", Red.bold().paint("Always"));
                println!("");
            }

            for path in right_paths.iter() {
            //for path in self.reduce_paths(sources, &right_paths) {
                self.print_path(sources, &path, "      ");
                println!("");
            }
        }
    }
}