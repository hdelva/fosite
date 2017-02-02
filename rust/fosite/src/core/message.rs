use super::GastID;
use super::Path;
use super::GastNode;


use std::collections::HashMap;

pub type MessageType = i16;

pub const EATTRIBUTE_INVALID: MessageType = 1;
pub const WATTRIBUTE_UNSAFE: MessageType = 2;
pub const NPROCESSING_NODE: MessageType = 3;
pub const NPROCESSED_NODE: MessageType = 4;
pub const EIDENTIFIER_INVALID: MessageType = 5;
pub const WIDENTIFIER_UNSAFE: MessageType = 6;
pub const WATTRIBUTE_POLY_TYPE: MessageType = 7;
pub const WIDENTIFIER_POLY_TYPE: MessageType = 8;
pub const EBINOP: MessageType = 9;

#[derive(Clone, Debug)]
pub enum Message {
    Error {
        source: GastID,
        kind: i16,
        content: HashMap<String, MessageItem>,
    },
    Warning {
        source: GastID,
        kind: i16,
        content: HashMap<String, MessageItem>,
    },
    Input {
        source: GastID,
        line: i16,
        col: i16,
        node: GastNode,
    },
    Notification {
        source: GastID,
        kind: i16,
        content: HashMap<String, MessageItem>,
    },
    Terminate,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MessageItem {
    Number(i16),
    String(String),
    Path(Path),
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

    pub fn to_path(&self) -> Option<Path> {
        match self {
            &MessageItem::Path(ref content) => Some(content.clone()),
            _ => None,
        }
    }
}
