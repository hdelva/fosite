use super::GastID;
use super::Path;
use super::GastNode;
use super::PathNode;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

use std::collections::HashMap;

use super::PathID;

type Sources = HashMap<GastID, (i16, i16)>;
type Nodes = HashMap<GastID, GastNode>;

pub enum Message {
    Output {
        source: PathID,
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
    fn hash(&self, source: &PathID) -> u64;

    fn print_warning_preamble(&self, sources: &Sources, node: PathID) {
        let source_node = node.last().unwrap();
        let &(row, col) = sources.get(source_node).unwrap();
        println!("{}",
            Custom(220).bold().paint(format!("Warning at row {}, column {}", row, col + 1)));
    }

    fn print_error_preamble(&self,  sources: &Sources, node: PathID) {
        let source_node = node.last().unwrap();
        let &(row, col) = sources.get(source_node).unwrap();
        println!("{}",
            Red.bold().paint(format!("Error at row {}, column {}", row, col + 1)));
    }

    fn print_path(&self, sources: &Sources, path: &Path, padding: &str) {
        if path.len() != 0 {
            for node in path.iter() {
                let row;
                let col;
                
                if let Some(source_node) = node.get_location().last(){
                    if let Some( &(pls1, pls2) ) = sources.get(&source_node) {
                        row = pls1;
                        col = pls2;
                    } else {
                        continue;
                    }                    
                } else {
                    continue;
                }
                

                match node {
                    &PathNode::Condition(_, b, _) => {
                        let condition = if b == 0 { "true" } else { "false" };
                        println!("{}{} {} is {}",
                                 padding,
                                 "Condition at",
                                 Bold.paint(format!("row {}, column {}", row, col + 1)),
                                 Bold.paint(format!("{}", condition)));
                    }
                    &PathNode::Loop(_, b, _) => {
                        let taken = if b == 0 { "executed" } else { "not executed" };
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

    fn print_message(&self, source: &Sources, nodes: &Nodes, node: PathID);
}