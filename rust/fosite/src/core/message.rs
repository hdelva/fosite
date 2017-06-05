use super::GastID;
use super::Path;
use super::GastNode;
use super::PathNode;

use std::collections::BTreeSet;

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
    #[allow(ptr_arg)]
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

    fn reduce_paths(&self, sources: &Sources, paths: &BTreeSet<Path>) -> BTreeSet<Path> {
        let mut result = BTreeSet::new();

        for path in paths {
            let mut new_path = Path::empty();
            let mut current_node: Option<&PathNode> = None;
            let mut current_col = 0;

            for node in path {           
                if let Some(source_node) = node.get_location().last(){
                    if let Some( &(_, col) ) = sources.get(source_node) {
                        if col <= current_col {
                            if let Some(acc) = current_node {
                                new_path.add_node(acc.clone());
                            }
                        } 
                        
                        if col < current_col {
                            current_node = None;
                        } else {
                            current_node = Some(node);
                        }

                        current_col = col;
                    }                   
                } 
            }

            if let Some(acc) = current_node {
                new_path.add_node(acc.clone());
            }

            result.insert(new_path);
        }

        result
    }

    fn print_path(&self, sources: &Sources, path: &Path, padding: &str) {
        if !path.is_empty() {
            for node in path {
                let row;
                let col;
                
                if let Some(source_node) = node.get_location().last(){
                    if let Some( &(pls1, pls2) ) = sources.get(source_node) {
                        row = pls1;
                        col = pls2;
                    } else {
                        continue;
                    }                    
                } else {
                    continue;
                }
                

                match *node {
                    PathNode::Condition(_, b, _) => {
                        let condition = if b == 0 { "true" } else { "false" };
                        println!("{}{} {} is {}",
                                 padding,
                                 "Condition at",
                                 Bold.paint(format!("row {}, column {}", row, col + 1)),
                                 Bold.paint(condition));
                    }
                    PathNode::Loop(_) => {
                        println!("{}Iteration of the loop at {}",
                                 padding,
                                 Bold.paint(format!("row {}, column {}", row, col + 1)));
                    }
                    PathNode::Assignment(_, ref name) => {
                        println!("{}Assignment to {} at {}",
                                 padding,
                                 Bold.paint(name),
                                 Bold.paint(format!("row {}, column {}", row, col + 1)));
                    }
                    PathNode::Return(_) => {
                        println!("{}{} {}",
                                 padding,
                                 "Return at",
                                 Bold.paint(format!("row {}, column {}", row, col + 1)));
                    }
                    PathNode::Element(_, _, _) => {
                        println!("{}{} {}",
                                 padding,
                                 "Element of the collection at",
                                 Bold.paint(format!("row {}, column {}", row, col + 1)));
                    }
                    PathNode::Frame(_, ref target, _, _) => {
                        println!("{}Call to {} at {}",
                                 padding,
                                 Bold.paint(target.as_ref().unwrap()),
                                 Bold.paint(format!("row {}, column {}", row, col + 1)));
                    }
                }
            }
        }
    }

    fn print_message(&self, source: &Sources, nodes: &Nodes, node: PathID);
}