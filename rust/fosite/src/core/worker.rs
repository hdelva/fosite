use super::message::*;
use super::CHANNEL;
use std::thread::*;
use std::collections::HashMap;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use super::GastID;
use super::{Path, PathNode};
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
		logger.add_error_handler(EBINOP, Box::new(BinopInvalid::new()));
    	
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
	
	
}

trait WarningHandler {
	fn preamble(&self, sources: &Sources, node: GastID)  {
		let &(row, col) = sources.get(&node).unwrap();
		println!("{}", Custom(220).bold().paint(format!("Warning at row {}, column {}", row, col+1)));
	}

	fn print_path(&self, sources: &Sources, path: &Path, padding: &str) {
		if path.len() != 0 {
			for node in path.iter() {
				let &(row, col) = sources.get(&node.get_location()).unwrap();

				match node {
					&PathNode::Condition(_, b) => {
						let condition = if b {"true"} else {"false"};
						println!("{}{} {} is {}", padding,
							"Condition at",
							Bold.paint(format!("row {}, column {}", row, col+1)),
							Bold.paint(format!("{}", condition)));
					},
					&PathNode::Loop(_, b) => {
						let taken = if b {"executed"} else {"not executed"};
						println!("{}{} {} is {}", padding,
							"Loop at",
							Bold.paint(format!("row {}, column {}", row, col+1)),
							Bold.paint(format!("{}", taken)));
					},
					&PathNode::Assignment(_, ref name) => {
						println!("{}Assignment to {} at {}", padding,
							Bold.paint(format!("{}", name)),
							Bold.paint(format!("row {}, column {}", row, col+1)));
					},
					&PathNode::Return(_) => {
						println!("{}{} {}", padding,
							"Return at",
							Bold.paint(format!("row {}, column {}", row, col+1)));
					},
					_ => {
						println!("Frame?");
					}
				}
			}
		}
	}	

	fn handle(&mut self, node: GastID, sources: &Sources,content: &Content);
}

struct AttributeUnsafe {
	done: BTreeSet<BTreeSet<BTreeSet<GastID>>>,
}

impl AttributeUnsafe {
	pub fn new() -> AttributeUnsafe {
		AttributeUnsafe {
			done: BTreeSet::new(),
		}
	}

	fn message_id(&self, content: &Content) -> BTreeSet<BTreeSet<GastID>> {
		let mut fingerprint = BTreeSet::new();

		let mut ass_count = 0;
		let mut current_ass = format!("path {}", ass_count);
		while let Some(path) = content.get(&current_ass) {
			let path = path.to_path().unwrap();

			let mut set = BTreeSet::new();
			for node in path.iter() {
				match node {
					&PathNode::Assignment(location, _) => {
						set.insert(location);
					},
					_ => (),
				}
			};

			fingerprint.insert(set);

			ass_count += 1;
			current_ass = format!("path {}", ass_count);
		}

		return fingerprint;
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
		let mut current_ass = format!("path {}", ass_count);
		while let Some(path) = content.get(&current_ass) {
			self.print_path(sources, &path.to_path().unwrap(), "    ");
			ass_count += 1;
			current_ass = format!("path {}", ass_count);
			println!("");
		}
	}
}

struct IdentifierUnsafe {
	done: BTreeSet<BTreeSet<BTreeSet<GastID>>>,
}

impl IdentifierUnsafe {
	pub fn new() -> IdentifierUnsafe {
		IdentifierUnsafe {
			done: BTreeSet::new(),
		}
	}

	fn message_id(&self, content: &Content) -> BTreeSet<BTreeSet<GastID>> {
		let mut fingerprint = BTreeSet::new();

		let mut ass_count = 0;
		let mut current_ass = format!("path {}", ass_count);
		while let Some(path) = content.get(&current_ass) {
			let path = path.to_path().unwrap();

			let mut set = BTreeSet::new();
			for node in path.iter() {
				match node {
					&PathNode::Assignment(location, _) => {
						set.insert(location);
					},
					_ => (),
				}
			};

			fingerprint.insert(set);

			ass_count += 1;
			current_ass = format!("path {}", ass_count);
		}

		return fingerprint;
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
		println!("  {} does not always exist at the of this", Bold.paint(name));

		let mut ass_count = 0;
		let mut current_ass = format!("path {}", ass_count);
		while let Some(path) = content.get(&current_ass) {
			self.print_path(sources, &path.to_path().unwrap(), "    ");
			ass_count += 1;
			current_ass = format!("path {}", ass_count);
			println!("");
		}
	}
}

struct PolyType {
	done: BTreeSet<BTreeSet<(i16, PathNode)>>,
}

impl PolyType {
	pub fn new() -> PolyType {
		PolyType {
			done: BTreeSet::new(),
		}
	}

	fn message_id(&self, content: &Content) -> BTreeSet<(i16, PathNode)> {
		let mut set = BTreeSet::new();

		let mut type_count = 0;
		let mut current_type = format!("type {}", type_count);
		while let Some(type_name) = content.get(&current_type) {
			let mut path_count = 0;
			let mut current_ass = format!("type {} path {}", type_count, path_count);
			while let Some(path) = content.get(&current_ass) {
				let path = path.to_path().unwrap();
				match path.iter().next_back() {
					Some(thing) => set.insert( (type_count as i16, thing.clone())),
					_ => set.insert( (0, PathNode::Condition(0, true)) ),
				};
				path_count += 1;
				current_ass = format!("type {} path {}", type_count, path_count);
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

			let mut path_count = 0;
			let mut current_ass = format!("type {} path {}", type_count, path_count);
			while let Some(path) = content.get(&current_ass) {
				self.print_path(sources, &path.to_path().unwrap(), "    ");
				path_count += 1;
				current_ass = format!("type {} path {}", type_count, path_count);
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

	fn print_path(&self, sources: &Sources, path: &Path, padding: &str) {
		if path.len() != 0 {
			for node in path.iter() {
				let &(row, col) = sources.get(&node.get_location()).unwrap();

				match node {
					&PathNode::Condition(_, b) => {
						let condition = if b {"true"} else {"false"};
						println!("{}{} {} is {}", padding,
							"Condition at",
							Bold.paint(format!("row {}, column {}", row, col+1)),
							Bold.paint(format!("{}", condition)));
					},
					&PathNode::Loop(_, b) => {
						let taken = if b {"executed"} else {"not executed"};
						println!("{}{} {} is {}", padding,
							"Loop at",
							Bold.paint(format!("row {}, column {}", row, col+1)),
							Bold.paint(format!("{}", taken)));
					},
					&PathNode::Assignment(_, ref name) => {
						println!("{}Assignment to {} at {}", padding,
							Bold.paint(format!("{}", name)),
							Bold.paint(format!("row {}, column {}", row, col+1)));
					},
					&PathNode::Return(_) => {
						println!("{}{} {}", padding,
							"Return at",
							Bold.paint(format!("row {}, column {}", row, col+1)));
					},
					_ => {
						println!("Frame?");
					}
				}
			}
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
		let mut current_ass = format!("path {}", ass_count);
		while let Some(path) = content.get(&current_ass) {
			self.print_path(sources, &path.to_path().unwrap(), "    ");
			ass_count += 1;
			current_ass = format!("path {}", ass_count);
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
		let mut current_ass = format!("path {}", ass_count);
		while let Some(path) = content.get(&current_ass) {
			self.print_path(sources, &path.to_path().unwrap(), "    ");
			ass_count += 1;
			current_ass = format!("path {}", ass_count);
			println!("");
		}
	}
}

struct BinopInvalid {
	
}

impl BinopInvalid {
	pub fn new() -> BinopInvalid {
		BinopInvalid {

		}
	}
}

impl ErrorHandler for BinopInvalid {
	fn handle(&mut self, node: GastID, sources: &Sources, nodes: &Nodes, content: &Content) {
		self.preamble(sources, node);

		let operator = content.get(&"operation".to_owned()).unwrap().to_string().unwrap();

		println!("  Incompatible types for operation {}", Bold.paint(operator.clone()));
		println!("  The following combinations exist:");

		let mut comb_count = 0;
		let mut current_left_comb = format!("combination {} left", comb_count);
		let mut current_right_comb = format!("combination {} right", comb_count);
		while let Some(left_type) = content.get(&current_left_comb) {
			let right_type = content.get(&current_right_comb).unwrap();
			let left_type = left_type.to_string().unwrap();
			let right_type = right_type.to_string().unwrap();

			println!("  Combination {}: {}", comb_count,
				Bold.paint(format!("{} {} {}", left_type, operator, right_type)));

			let mut ass_count = 0;
			let mut current_left_ass = format!("combination {} left {}", comb_count, ass_count);
			println!("    Left side has type {}", left_type);
			while let Some(left_ass) = content.get(&current_left_ass) {
				self.print_path(sources, &left_ass.to_path().unwrap(), "      ");
				println!("");

				ass_count += 1;
				current_left_ass = format!("combination {} left {}", comb_count, ass_count);
			}

			let mut ass_count = 0;
			let mut current_right_ass = format!("combination {} right {}", comb_count, ass_count);
			println!("    Right side has type {}", right_type);
			while let Some(right_ass) = content.get(&current_right_ass) {
				self.print_path(sources, &right_ass.to_path().unwrap(), "      ");
				println!("");

				ass_count += 1;
				current_right_ass = format!("combination {} right {}", comb_count, ass_count);
			}
			
			comb_count += 1;
			current_left_comb = format!("combination {} left", comb_count);
			current_right_comb = format!("combination {} right", comb_count);
		}
	}
}