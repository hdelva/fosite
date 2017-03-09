use super::GastID;
use super::Path;
use super::GastNode;
use super::PathNode;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use std::collections::HashMap;

type Sources = HashMap<GastID, (i16, i16)>;
type Nodes = HashMap<GastID, GastNode>;

pub enum Message {
    Output {
        source: GastID,
        content: Box<MessageContent>,
    },
    Input {
        source: GastID,
        line: i16,
        col: i16,
        node: GastNode,
    },
    Terminate,
}

pub trait MessageContent: Send {
    fn hash(&self) -> u64;

    fn print_warning_preamble(&self, sources: &Sources, node: GastID) {
        let &(row, col) = sources.get(&node).unwrap();
        println!("{}",
            Custom(220).bold().paint(format!("Warning at row {}, column {}", row, col + 1)));
    }

    fn print_error_preamble(&self,  sources: &Sources, node: GastID) {
        let &(row, col) = sources.get(&node).unwrap();
        println!("{}",
            Red.bold().paint(format!("Error at row {}, column {}", row, col + 1)));
    }

    fn print_path(&self, sources: &Sources, path: &Path, padding: &str) {
        if path.len() != 0 {
            for node in path.iter() {
                let &(row, col) = sources.get(&node.get_location()).unwrap();

                match node {
                    &PathNode::Condition(_, b) => {
                        let condition = if b { "true" } else { "false" };
                        println!("{}{} {} is {}",
                                 padding,
                                 "Condition at",
                                 Bold.paint(format!("row {}, column {}", row, col + 1)),
                                 Bold.paint(format!("{}", condition)));
                    }
                    &PathNode::Loop(_, b) => {
                        let taken = if b { "executed" } else { "not executed" };
                        println!("{}{} {} is {}",
                                 padding,
                                 "Loop at",
                                 Bold.paint(format!("row {}, column {}", row, col + 1)),
                                 Bold.paint(format!("{}", taken)));
                    }
                    &PathNode::Assignment(_, ref name) => {
                        println!("{}Assignment to {} at {}",
                                 padding,
                                 Bold.paint(format!("{}", name)),
                                 Bold.paint(format!("row {}, column {}", row, col + 1)));
                    }
                    &PathNode::Return(_) => {
                        println!("{}{} {}",
                                 padding,
                                 "Return at",
                                 Bold.paint(format!("row {}, column {}", row, col + 1)));
                    }
                    &PathNode::Element(_, _, _) => {
                        println!("{}{} {}",
                                 padding,
                                 "Element of the collection at",
                                 Bold.paint(format!("row {}, column {}", row, col + 1)));
                    }
                    _ => {
                        println!("Frame?");
                    }
                }
            }
        }
    }

    fn print_message(&self, source: &Sources, nodes: &Nodes, node: GastID);
}