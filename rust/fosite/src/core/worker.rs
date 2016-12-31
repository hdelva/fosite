use super::Message;
use super::CHANNEL;
use std::thread::*;
use std::collections::HashMap;
use super::GastID;
use super::Assumption;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

pub struct Worker {
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new() -> Worker {
    	let mut logger = Logger::new();
    	
        let thread = {
            spawn(move || 
            	logger.message_loop()
            )
        };

        let worker = Worker {
            thread: thread,
        };

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
	sources: HashMap<GastID, (i16, i16)>,
}

impl Logger {
	fn new() -> Logger {
		Logger {
			sources: HashMap::new(),
		}
	}
	
	fn message_loop(&mut self) {
		for message in CHANNEL.iter() {
            match message {
            	Message::Error { ref source, ref assumption, ref content } => self.print_error(source, assumption, content),
            	Message::Warning { ref source, ref assumption, ref content } => self.print_warning(source, assumption, content),
                Message::Notification { ref source, ref content } => {
                    //println!("Message from node {} at {:?}: {}", source, self.sources.get(source), content)
                },
                Message::Input {source, line, col} => {
                	//println!("mapping node {} to ({}, {})", source, line, col);
                	self.sources.insert(source, (line, col));
                },
                Message::Terminate => break,
            }
        }
		
		
	}
	
	fn print_warning(&self, source: &GastID, assumption: &Assumption, content: &String) {
		let &(row, col) = self.sources.get(source).unwrap();
		println!("{}", Custom(220).bold().paint(format!("Warning at row {}, column {}", row, col+1)));
		self.print_assumption(assumption);
        println!("{:?}\n", content)
	}
	
	fn print_error(&self, source: &GastID, assumption: &Assumption, content: &String) {
		let &(row, col) = self.sources.get(source).unwrap();
		println!("{}", Red.bold().paint(format!("Error at row {}, column {}", row, col+1)));
		self.print_assumption(assumption);
        println!("{:?}\n", content)
	}
	
	fn print_assumption(&self, assumption: &Assumption) {
		println!("{}", Bold.paint("Under the following assumptions:"));
		for &(source, positive) in assumption.iter() {
			let &(row, col) = self.sources.get(&source).unwrap();
			let condition = if positive {"true"} else {"false"};
			println!("  {} {} is {}", "Condition at",
									  Bold.paint(format!("row {}, column {}", row, col+1)),
									  Bold.paint(format!("{}", condition)));
		}
	}
	
	
}
