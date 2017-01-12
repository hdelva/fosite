use super::message::*;
use super::CHANNEL;
use std::thread::*;
use std::collections::HashMap;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use super::GastID;
use super::Assumption;
use super::GastNode;
use super::NodeType;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

const DEBUG: bool = false;

type Sources = HashMap<GastID, (i16, i16)>;

pub struct Worker {
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new() -> Worker {
    	let mut logger = Logger::new();
		logger.add_warning_handler(WATTRIBUTE_UNSAFE, Box::new(AttributeUnsafe::new()));
		logger.add_warning_handler(WIDENTIFIER_UNSAFE, Box::new(IdentifierUnsafe::new()));
		logger.add_warning_handler(WIDENTIFIER_POLY_TYPE, Box::new(PolyType::new()));
		logger.add_warning_handler(WATTRIBUTE_POLY_TYPE, Box::new(PolyType::new()));
    	
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
	nodes: HashMap<GastID, GastNode>,
	sources: Sources,
	warning_handlers: BTreeMap<i16, Box<WarningHandler + Send>>,
}

impl Logger {
	fn new() -> Logger {
		Logger {
			sources: HashMap::new(),
			nodes: HashMap::new(),
			warning_handlers: BTreeMap::new(),
		}
	}

	fn add_warning_handler(&mut self, number: i16, handler: Box<WarningHandler + Send>) {
		self.warning_handlers.insert(number, handler);
	}
	
	fn message_loop(&mut self) {
		for message in CHANNEL.iter() {
            match message {
            	Message::Error { ref source, ref kind, ref content } => self.print_error(source, kind, content),
            	Message::Warning { ref source, ref kind, ref content } => {
					let mut opt_handler = self.warning_handlers.get_mut(kind);
					if let Some(handler) = opt_handler {
						handler.handle(*source, &self.sources, content);
					} else {
						println!("  Unknown Warning: {:?}\n", message);
					}
				},
                Message::Notification { ref source, ref kind, ref content } => {
                    self.print_notification(source, kind, content);
                },
                Message::Input {source, line, col, node} => {
                	//println!("mapping node {} to ({}, {})", source, line, col);
                	self.sources.insert(source, (line, col));
					self.nodes.insert(source, node);
                },
                Message::Terminate => break,
            }
        }
	}
	
	fn print_notification(&self, source: &GastID, kind: &i16, content: &HashMap<String, MessageItem>) {	
		if !DEBUG {
			return
		}

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
	
	fn print_error(&mut self, source: &GastID, kind: &i16, content: &HashMap<String, MessageItem>) {
		match kind {
			&EATTRIBUTE_INVALID => {
				match self.nodes.get(source).unwrap() {
					&GastNode { kind: NodeType::Attribute {..}, .. } => (),
					_ => return,
				}
			},
			&EIDENTIFIER_INVALID => {
				match self.nodes.get(source).unwrap() {
					&GastNode { kind: NodeType::Identifier {..}, .. } => (),
					_ => return,
				}
			},
			_ => (),
		};

		let &(row, col) = self.sources.get(source).unwrap();
		println!("{}", Red.bold().paint(format!("Error at row {}, column {}", row, col+1)));
		let assumption = content.get(&"assumption".to_owned()).unwrap().to_assumption().unwrap();
		self.print_assumption(&assumption, "  ");
		
        let message = match kind {
			&EATTRIBUTE_INVALID => "Object does not have an attribute of this name",
			&EIDENTIFIER_INVALID => "An identifier of this name does not exist",
			_ => "Unknown error",
		};
		
        println!("  {:?}\n", message)
	}
	
	fn print_assumption(&self, assumption: &Assumption, padding: &str) {
		println!("{:?}", assumption);
		if assumption.len() != 0 {
			println!("{}{}", padding, Bold.paint("Under the following assumptions:"));
			for &(source, positive) in assumption.iter() {
				let &(row, col) = self.sources.get(&source).unwrap();
				let condition = if positive {"true"} else {"false"};
				println!("{}{} {} is {}", padding,
										"Condition at",
										Bold.paint(format!("row {}, column {}", row, col+1)),
										Bold.paint(format!("{}", condition)));
			}
		}
	}
	
	
}

trait WarningHandler {
	fn preamble(&self, sources: &Sources, node: GastID)  {
		let &(row, col) = sources.get(&node).unwrap();
		println!("{}", Custom(220).bold().paint(format!("Warning at row {}, column {}", row, col+1)));
	}

	fn print_assumption(&self, sources: &Sources, assumption: &Assumption, padding: &str) {
		println!("{:?}", assumption);
		if assumption.len() != 0 {
			println!("{}{}", padding, Bold.paint("Under the following assumptions:"));
			for &(source, positive) in assumption.iter() {
				let &(row, col) = sources.get(&source).unwrap();
				let condition = if positive {"true"} else {"false"};
				println!("{}{} {} is {}", padding,
										"Condition at",
										Bold.paint(format!("row {}, column {}", row, col+1)),
										Bold.paint(format!("{}", condition)));
			}
		}
	}

	fn handle(&mut self, node: GastID, sources: &Sources,content: &HashMap<String, MessageItem>);
}

struct AttributeUnsafe {
	done: BTreeSet< (GastID, bool) >,
}

impl AttributeUnsafe {
	pub fn new() -> AttributeUnsafe {
		AttributeUnsafe {
			done: BTreeSet::new(),
		}
	}

	fn message_id(&self, content: &HashMap<String, MessageItem>) -> (GastID, bool) {
		let assumption = content.get(&"assumption".to_owned()).unwrap().to_assumption().unwrap();

		match assumption.get().iter().next_back() {
			Some(thing) => return thing.clone(),
			_ => return (0, true),
		}
	}
}

impl WarningHandler for AttributeUnsafe {
	fn handle(&mut self, node: GastID, sources: &Sources,content: &HashMap<String, MessageItem>) {
		let identifier = self.message_id(content);
		if self.done.contains(&identifier) {
			return;			
		}

		self.done.insert(identifier);

		self.preamble(sources, node);
		let assumption = content.get(&"assumption".to_owned()).unwrap().to_assumption().unwrap();
		self.print_assumption(sources, &assumption, "  ");
		println!("  {:?}\n", "Object does not always have an attribute of this name");
	}
}

struct IdentifierUnsafe {
	done: BTreeSet< (GastID, bool) >,
}

impl IdentifierUnsafe {
	pub fn new() -> IdentifierUnsafe {
		IdentifierUnsafe {
			done: BTreeSet::new(),
		}
	}

	fn message_id(&self, content: &HashMap<String, MessageItem>) -> (GastID, bool) {
		let assumption = content.get(&"assumption".to_owned()).unwrap().to_assumption().unwrap();

		match assumption.get().iter().next_back() {
			Some(thing) => return thing.clone(),
			_ => return (0, true),
		}
	}
}

impl WarningHandler for IdentifierUnsafe {
	fn handle(&mut self, node: GastID, sources: &Sources,content: &HashMap<String, MessageItem>) {
		let identifier = self.message_id(content);
		if self.done.contains(&identifier) {
			return;			
		}

		self.done.insert(identifier);

		self.preamble(sources, node);
		let assumption = content.get(&"assumption".to_owned()).unwrap().to_assumption().unwrap();
		self.print_assumption(sources, &assumption, "  ");
		println!("  {:?}\n", "An identifier of this name does not always exist");
	}
}

struct PolyType {
	done: BTreeSet<BTreeSet<MessageItem>>,
}

impl PolyType {
	pub fn new() -> PolyType {
		PolyType {
			done: BTreeSet::new(),
		}
	}

	fn message_id(&self, content: &HashMap<String, MessageItem>) -> BTreeSet<MessageItem> {
		let mut set = BTreeSet::new();
		for item in content.values() {
			set.insert(item.clone());
		}

		return set
	}
}

impl WarningHandler for PolyType {
	fn handle(&mut self, node: GastID, sources: &Sources, content: &HashMap<String, MessageItem>) {
		let identifier = self.message_id(content);
		if self.done.contains(&identifier) {
			return;			
		}

		self.done.insert(identifier);

		self.preamble(sources, node);

		let name = content.get(&"name".to_owned()).unwrap().to_string().unwrap();
		println!("  Identifier {} does not always have the same type after executing this", Bold.paint(name));

		let mut type_count = 0;
		let mut current_type = format!("type {}", type_count);
		while let Some(type_name) = content.get(&current_type) {
			println!("  Type {}: {}", type_count, Bold.paint(type_name.to_string().unwrap()));

			let mut ass_count = 0;
			let mut current_ass = format!("type {} assumption {}", type_count, ass_count);
			while let Some(assumption) = content.get(&current_ass) {
				self.print_assumption(sources, &assumption.to_assumption().unwrap(), "    ");
				ass_count += 1;
				current_ass = format!("type {} assumption {}", type_count, ass_count);
				println!("");
			}

			type_count += 1;
			current_type = format!("type {}", type_count);
		}
	}
}