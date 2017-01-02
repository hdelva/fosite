use super::message::*;
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
            	Message::Error { ref source, ref kind, ref content } => self.print_error(source, kind, content),
            	Message::Warning { ref source, ref kind, ref content } => self.print_warning(source, kind, content),
                Message::Notification { ref source, ref kind, ref content } => {
                    self.print_notification(source, kind, content);
                },
                Message::Input {source, line, col} => {
                	//println!("mapping node {} to ({}, {})", source, line, col);
                	self.sources.insert(source, (line, col));
                },
                Message::Terminate => break,
            }
        }
	}
	
	fn print_notification(&self, source: &GastID, kind: &i16, content: &HashMap<String, MessageItem>) {		
		if let Some( &(row, col) ) = self.sources.get(source) {
			println!("{}", Custom(112).bold().paint(format!("Notification from row {}, column {}", row, col+1)));
		} else {
			return;
		}
		
		match kind {
			&NPROCESSED_NODE => {
				println!("  Processed node {}, this is the result:", source);
				println!("  {:?}\n", content.get("node").unwrap().to_string().unwrap());
			},
			&NPROCESSING_NODE => {
				println!("  Processing node {}:", source);
				println!("  {:?}\n", content.get("node").unwrap().to_string().unwrap());
			},
			_ => println!("  Unknown Notification\n"),
		};
	}
	
	fn print_warning(&self, source: &GastID, kind: &i16, content: &HashMap<String, MessageItem>) {
		let &(row, col) = self.sources.get(source).unwrap();
		println!("{}", Custom(220).bold().paint(format!("Warning at row {}, column {}", row, col+1)));
		let assumption = content.get(&"assumption".to_owned()).unwrap().to_assumption().unwrap();
		self.print_assumption(&assumption);
		
		let message = match kind {
			&WATTRIBUTE_UNSAFE => "Object does not always have an attribute of this name",
			_ => "Unknown warning",
		};
		
        println!("  {:?}\n", message)
	}
	
	fn print_error(&self, source: &GastID, kind: &i16, content: &HashMap<String, MessageItem>) {
		let &(row, col) = self.sources.get(source).unwrap();
		println!("{}", Red.bold().paint(format!("Error at row {}, column {}", row, col+1)));
		let assumption = content.get(&"assumption".to_owned()).unwrap().to_assumption().unwrap();
		self.print_assumption(&assumption);
		
        let message = match kind {
			&EATTRIBUTE_INVALID => "Object does not have an attribute of this name",
			_ => "Unknown error",
		};
		
        println!("  {:?}\n", message)
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
