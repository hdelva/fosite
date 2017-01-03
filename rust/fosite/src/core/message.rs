use super::GastID;
use super::Assumption;
use super::GastNode;


use std::collections::HashMap;

pub type message_type = i16;

pub const EATTRIBUTE_INVALID: message_type = 1;
pub const WATTRIBUTE_UNSAFE: message_type = 2;
pub const NPROCESSING_NODE: message_type = 3;
pub const NPROCESSED_NODE: message_type = 4;
pub const EIDENTIFIER_INVALID: message_type = 5;
pub const WIDENTIFIER_UNSAFE: message_type = 6;
pub const WATTRIBUTE_POLY_TYPE: message_type = 7;
pub const WIDENTIFIER_POLY_TYPE: message_type = 8;

#[derive(Clone, Debug)]
pub enum Message {
	Error {source: GastID, kind: i16, content: HashMap<String, MessageItem> },
	Warning {source: GastID, kind: i16, content: HashMap<String, MessageItem> },
	Input { source: GastID, line: i16, col: i16, node: GastNode },
    Notification { source: GastID, kind: i16, content: HashMap<String, MessageItem> },
    Terminate,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MessageItem {
	Number(i16),
	String(String),
	Assumption(Assumption),
}

impl MessageItem {
	pub fn to_number(&self) -> Option<i16> {
		match self {
			&MessageItem::Number(content) => Some(content),
			_ => None,
		}
	}
	
	pub fn to_string(&self) -> Option<String> {
		match self {
			&MessageItem::String(ref content) => Some(content.clone()),
			_ => None,
		}
	}
	
	pub fn to_assumption(&self) -> Option<Assumption> {
		match self {
			&MessageItem::Assumption(ref content) => Some(content.clone()),
			_ => None,
		}
	}
}


