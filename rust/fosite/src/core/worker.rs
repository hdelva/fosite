use super::message::*;
use super::CHANNEL;
use std::thread::*;
use std::collections::HashMap;
use std::collections::HashSet;
use super::GastID;
use super::GastNode;

type Sources = HashMap<GastID, (i16, i16)>;
type Nodes = HashMap<GastID, GastNode>;

pub struct Worker {
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new() -> Worker {
        let mut logger = Logger::new();

        let thread = {
            spawn(move || logger.message_loop())
        };

        let worker = Worker { thread: thread };

        return worker;
    }

    pub fn finalize(self) -> Result<()> {
        &CHANNEL.publish({
            Message::Terminate
        });
        return self.thread.join();
    }
}


struct Logger {
    nodes: Nodes,
    sources: Sources,
    done: HashSet<u64>,
}

impl Logger {
    fn new() -> Logger {
        Logger {
            sources: HashMap::new(),
            nodes: HashMap::new(),
            done: HashSet::new(),
        }
    }

    fn message_loop(&mut self) {
        for message in CHANNEL.iter() {
            match message {
                Message::Output { ref source, ref content } => {
                    if !self.done.contains(&content.hash(source)) {
                        self.done.insert(content.hash(source));
                        content.print_message(&self.sources, &self.nodes, source.clone());
                    }
                }
                Message::Input { source, line, col, node } => {
                    // println!("mapping node {} to ({}, {})", source, line, col);
                    self.sources.insert(source, (line, col));
                    self.nodes.insert(source, node);
                }
                Message::Terminate => break,
            }
        }
    }
}