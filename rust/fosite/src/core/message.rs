use super::GastID;
use super::Assumption;

use std::collections::HashMap;

pub const EATTRIBUTE_INVALID: i16 = 1;
pub const WATTRIBUTE_UNSAFE: i16 = 2;
pub const NPROCESSING_NODE: i16 = 3;
pub const NPROCESSED_NODE: i16 = 4;

#[derive(Clone, Debug)]
pub enum Message {
	Error {source: GastID, kind: i16, content: HashMap<String, MessageItem> },
	Warning {source: GastID, kind: i16, content: HashMap<String, MessageItem> },
	Input { source: GastID, line: i16, col: i16 },
    Notification { source: GastID, kind: i16, content: HashMap<String, MessageItem> },
    Terminate,
}

#[derive(Clone, Debug)]
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


