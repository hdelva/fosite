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
type Nodes = HashMap<GastID, GastNode>;
type Content = HashMap<String, MessageItem>;

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

		logger.add_error_handler(EATTRIBUTE_INVALID, Box::new(AttributeInvalid::new()));
		logger.add_error_handler(EIDENTIFIER_INVALID, Box::new(IdentifierInvalid::new()));
    	
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
	nodes: Nodes,
	sources: Sources,
	warning_handlers: BTreeMap<i16, Box<WarningHandler + Send>>,
	error_handlers: BTreeMap<i16, Box<ErrorHandler + Send>>,
}

impl Logger {
	fn new() -> Logger {
		Logger {
			sources: HashMap::new(),
			nodes: HashMap::new(),
			warning_handlers: BTreeMap::new(),
			error_handlers: BTreeMap::new(),
		}
	}

	fn add_warning_handler(&mut self, number: i16, handler: Box<WarningHandler + Send>) {
		self.warning_handlers.insert(number, handler);
	}

	fn add_error_handler(&mut self, number: i16, handler: Box<ErrorHandler + Send>) {
		self.error_handlers.insert(number, handler);
	}
	
	fn message_loop(&mut self) {
		for message in CHANNEL.iter() {
            match message {
            	Message::Error { ref source, ref kind, ref content } => {
					let mut opt_handler = self.error_handlers.get_mut(kind);
					if let Some(handler) = opt_handler {
						handler.handle(*source, &self.sources, &self.nodes, content);
					} else {
						println!("  Unknown Error: {:?}\n", message);
					}
				},
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
		} else {
			println!("{}{}", padding, Red.bold().paint("Under all circumstances"));
		}
	}

	fn handle(&mut self, node: GastID, sources: &Sources,content: &Content);
}

struct AttributeUnsafe {
	done: BTreeSet<BTreeSet<(GastID, bool)>>,
}

impl AttributeUnsafe {
	pub fn new() -> AttributeUnsafe {
		AttributeUnsafe {
			done: BTreeSet::new(),
		}
	}

	fn message_id(&self, content: &Content) -> BTreeSet<(GastID, bool)> {
		let mut set = BTreeSet::new();

		let mut ass_count = 0;
		let mut current_ass = format!("assumption {}", ass_count);
		while let Some(assumption) = content.get(&current_ass) {
			let assumption = assumption.to_assumption().unwrap();
			match assumption.get().iter().next_back() {
				Some(thing) => set.insert(thing.clone()),
				_ => set.insert((0, true)),
			};
			ass_count += 1;
			current_ass = format!("assumption {}", ass_count);
		}

		return set;
	}
}

impl WarningHandler for AttributeUnsafe {
	fn handle(&mut self, node: GastID, sources: &Sources, content: &Content) {
		let identifier = self.message_id(content);
		if self.done.contains(&identifier) {
			return;			
		}

		self.done.insert(identifier);

		self.preamble(sources, node);
		let parent = content.get(&"parent".to_owned()).unwrap().to_string().unwrap();
		let name = content.get(&"name".to_owned()).unwrap().to_string().unwrap();
		println!("  Object {} does not always have an attribute {}", Bold.paint(parent), Bold.paint(name));

		let mut ass_count = 0;
		let mut current_ass = format!("assumption {}", ass_count);
		while let Some(assumption) = content.get(&current_ass) {
			self.print_assumption(sources, &assumption.to_assumption().unwrap(), "    ");
			ass_count += 1;
			current_ass = format!("assumption {}", ass_count);
			println!("");
		}
	}
}

struct IdentifierUnsafe {
	done: BTreeSet<BTreeSet<(GastID, bool)>>,
}

impl IdentifierUnsafe {
	pub fn new() -> IdentifierUnsafe {
		IdentifierUnsafe {
			done: BTreeSet::new(),
		}
	}

	fn message_id(&self, content: &Content) -> BTreeSet<(GastID, bool)> {
		let mut set = BTreeSet::new();

		let mut ass_count = 0;
		let mut current_ass = format!("assumption {}", ass_count);
		while let Some(assumption) = content.get(&current_ass) {
			let assumption = assumption.to_assumption().unwrap();
			match assumption.get().iter().next_back() {
				Some(thing) => set.insert(thing.clone()),
				_ => set.insert((0, true)),
			};
			ass_count += 1;
			current_ass = format!("assumption {}", ass_count);
		}

		return set;
	}
}

impl WarningHandler for IdentifierUnsafe {
	fn handle(&mut self, node: GastID, sources: &Sources, content: &Content) {
		let identifier = self.message_id(content);
		if self.done.contains(&identifier) {
			return;			
		}

		self.done.insert(identifier);

		self.preamble(sources, node);
		let name = content.get(&"name".to_owned()).unwrap().to_string().unwrap();
		println!("  {} does not always exist", Bold.paint(name));

		let mut ass_count = 0;
		let mut current_ass = format!("assumption {}", ass_count);
		while let Some(assumption) = content.get(&current_ass) {
			self.print_assumption(sources, &assumption.to_assumption().unwrap(), "    ");
			ass_count += 1;
			current_ass = format!("assumption {}", ass_count);
			println!("");
		}
	}
}

struct PolyType {
	done: BTreeSet<BTreeSet<(i16, (GastID, bool))>>,
}

impl PolyType {
	pub fn new() -> PolyType {
		PolyType {
			done: BTreeSet::new(),
		}
	}

	fn message_id(&self, content: &Content) -> BTreeSet<(i16, (GastID, bool))> {
		let mut set = BTreeSet::new();

		let mut type_count = 0;
		let mut current_type = format!("type {}", type_count);
		while let Some(type_name) = content.get(&current_type) {
			let mut ass_count = 0;
			let mut current_ass = format!("type {} assumption {}", type_count, ass_count);
			while let Some(assumption) = content.get(&current_ass) {
				let assumption = assumption.to_assumption().unwrap();
				match assumption.get().iter().next_back() {
					Some(thing) => set.insert( (type_count as i16, thing.clone())),
					_ => set.insert( (0, (0, true)) ),
				};
				ass_count += 1;
				current_ass = format!("type {} assumption {}", type_count, ass_count);
			}

			type_count += 1;
			current_type = format!("type {}", type_count);
		}

		return set
	}
}

impl WarningHandler for PolyType {
	fn handle(&mut self, node: GastID, sources: &Sources, content: &Content) {
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

trait ErrorHandler {
	fn preamble(&self, sources: &Sources, node: GastID)  {
		let &(row, col) = sources.get(&node).unwrap();
		println!("{}", Red.bold().paint(format!("Error at row {}, column {}", row, col+1)));
	}

	fn print_assumption(&self, sources: &Sources, assumption: &Assumption, padding: &str) {
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
		} else {
			println!("{}{}", padding, Red.bold().paint("Under all circumstances\n"));
		}
	}

	fn handle(&mut self, node: GastID, sources: &Sources, nodes: &Nodes, content: &Content);
}

struct IdentifierInvalid {
	
}

impl IdentifierInvalid {
	pub fn new() -> IdentifierInvalid {
		IdentifierInvalid {

		}
	}
}

impl ErrorHandler for IdentifierInvalid {
	fn handle(&mut self, node: GastID, sources: &Sources, nodes: &Nodes, content: &Content) {
		match nodes.get(&node).unwrap() {
			&GastNode { kind: NodeType::Identifier {..}, .. } => (),
			_ => return,
		}

		self.preamble(sources, node);

		let name = content.get(&"name".to_owned()).unwrap().to_string().unwrap();
		println!("  {} does not exist", Bold.paint(name));

		let mut ass_count = 0;
		let mut current_ass = format!("assumption {}", ass_count);
		while let Some(assumption) = content.get(&current_ass) {
			self.print_assumption(sources, &assumption.to_assumption().unwrap(), "    ");
			ass_count += 1;
			current_ass = format!("assumption {}", ass_count);
			println!("");
		}
	}
}

struct AttributeInvalid {
	
}

impl AttributeInvalid {
	pub fn new() -> AttributeInvalid {
		AttributeInvalid {

		}
	}
}

impl ErrorHandler for AttributeInvalid {
	fn handle(&mut self, node: GastID, sources: &Sources, nodes: &Nodes, content: &Content) {
		match nodes.get(&node).unwrap() {
			&GastNode { kind: NodeType::Attribute {..}, .. } => (),
			_ => return,
		}

		self.preamble(sources, node);

		let parent = content.get(&"parent".to_owned()).unwrap().to_string().unwrap();
		let name = content.get(&"name".to_owned()).unwrap().to_string().unwrap();

		println!("  Object {} does not have an attribute {}", Bold.paint(parent), Bold.paint(name));

		let mut ass_count = 0;
		let mut current_ass = format!("assumption {}", ass_count);
		while let Some(assumption) = content.get(&current_ass) {
			self.print_assumption(sources, &assumption.to_assumption().unwrap(), "    ");
			ass_count += 1;
			current_ass = format!("assumption {}", ass_count);
			println!("");
		}
	}
}