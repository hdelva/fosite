use rustc_serialize::json::*;

use super::Message;
use super::CHANNEL;

pub type GastID = u16;

#[derive(Debug, Clone)]
pub struct GastNode {
    pub id: GastID,
    pub kind: NodeType,
}

impl GastNode {
    pub fn new(id: GastID, kind: NodeType) -> GastNode {
        GastNode {
            id: id,
            kind: kind,
        }
    }

    pub fn to_string(&self) -> String {
        self.kind.to_string()
    }
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Identifier { name: String },
    Attribute {
        parent: Box<GastNode>,
        attribute: String,
    },
    Declaration { id: String, kind: String },
    Assignment {
        targets: Vec<GastNode>,
        value: Box<GastNode>,
    },
    Int { value: i64 },
    Float { value: f64 },
    String { value: String },
    List { content: Vec<GastNode> },
    Set { content: Vec<GastNode> },
    Sequence { content: Vec<GastNode> },
    Block { content: Vec<GastNode> },
    If {
        test: Box<GastNode>,
        body: Box<GastNode>,
        or_else: Box<GastNode>,
    },
    While {
        test: Box<GastNode>,
        body: Box<GastNode>,
    },
    BinOp {
        left: Box<GastNode>,
        right: Box<GastNode>,
        op: String,
        associative: bool,
    },
    BoolOp {
        left: Box<GastNode>,
        op: String,
        right: Box<GastNode>,
        reversed: Option<String>,
        negated: Option<String>,
    },
    Break { },
    Continue { },
    Boolean { value: bool },
    Nil {},
    UnOp { op: String, value: Box<GastNode> },
    Index {
        target: Box<GastNode>,
        index: Box<GastNode>,
    }
}

impl NodeType {
    fn to_string(&self) -> String {
        match self {
            &NodeType::Identifier { ref name } => name.clone(),
            &NodeType::Attribute { ref parent, ref attribute } => {
                format!("{}.{}", parent.kind.to_string(), attribute)
            }
            &NodeType::Index { ref target, ref index } => {
                format!("{}[{}]", target.to_string(), index.to_string())
            }
            &NodeType::Int {ref value} => {
                format!("{}", value)
            }
            &NodeType::Float {ref value} => {
                format!("{}", value)
            }
            &NodeType::String {ref value} => {
                format!("{}", value)
            }
            _ => format!("Node {:?} doesn't have a string representation", self),
        }
    }
}

pub fn build(node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let kind = obj.get("kind").unwrap();
    let kind = kind.as_string().unwrap();
    let id = obj.get("id").unwrap();
    let id = id.as_i64().unwrap() as u16;

    let line = obj.get("line");
    let col = obj.get("col");

    let node = match kind {
        "block" => build_block(id, obj.get("content").unwrap()),
        "assign" => build_assign(id, node),
        "identifier" => build_identifier(id, node),
        "int" => build_int(id, node),
        "float" => build_float(id, node),
        "string" => build_string(id, node),
        "attribute" => build_attribute(id, node),
        "list" => build_list(id, node),
        "sequence" => build_sequence(id, node),
        "if" => build_if(id, node),
        "while" => build_while(id, node),
        "binop" => build_binop(id, node),
        "nil" => build_nil(id),
        "boolean" => build_bool(id, node),
        "boolop" => build_boolop(id, node),
        "break" => build_break(id),
        "continue" => build_continue(id),
        "unop" => build_unop(id, node),
        "index" => build_index(id, node),
        "set" => build_set(id, node),
        _ => panic!("unsupported JSON node: {:?}", node),
    };

    if let (Some(line), Some(col)) = (line, col) {
        let message = Message::Input {
            source: id.clone(),
            line: line.as_i64().unwrap() as i16,
            col: col.as_i64().unwrap() as i16,
            node: node.clone(),
        };

        &CHANNEL.publish(message);
    }

    return node;
}

fn build_binop(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_left = obj.get("left").unwrap();
    let left = Box::new(build(json_left));

    let json_right = obj.get("right").unwrap();
    let right = Box::new(build(json_right));

    let json_op = obj.get("op").unwrap();
    let op = json_op.as_string().unwrap().to_owned();

    let json_ass = obj.get("associative").unwrap();
    let ass = json_ass.as_boolean().unwrap();

    return GastNode::new(id,
                         NodeType::BinOp {
                             left: left,
                             right: right,
                             op: op,
                             associative: ass,
                         });
}

fn build_index(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_target = obj.get("target").unwrap();
    let target = Box::new(build(json_target));

    let json_index = obj.get("index").unwrap();
    let index = Box::new(build(json_index));

    return GastNode::new(id,
                         NodeType::Index {
                             target: target,
                             index: index,
                         });
}

fn build_unop(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_value = obj.get("value").unwrap();
    let value = Box::new(build(json_value));

    let json_op = obj.get("op").unwrap();
    let op = json_op.as_string().unwrap().to_owned();

    return GastNode::new(id,
                         NodeType::UnOp {
                             value: value,
                             op: op,
                         });
}

fn build_boolop(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_left = obj.get("left").unwrap();
    let left = Box::new(build(json_left));

    let json_right = obj.get("right").unwrap();
    let right = Box::new(build(json_right));

    let json_op = obj.get("op").unwrap();
    let op = json_op.as_string().unwrap().to_owned();

    let json_reversed = obj.get("reverse").unwrap();
    let reversed = if !json_reversed.is_null() {
        Some(json_reversed.as_string().unwrap().to_owned())
    } else {
        None
    };

    let json_negated = obj.get("negate").unwrap();
    let negated = if !json_negated.is_null() {
        Some(json_negated.as_string().unwrap().to_owned())
    } else {
        None
    };

    return GastNode::new(id,
                         NodeType::BoolOp {
                             left: left,
                             right: right,
                             op: op,
                             reversed: reversed,
                             negated: negated,
                         });
}

fn build_bool(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_value = obj.get("value").unwrap();
    let value = json_value.as_boolean().unwrap();

    return GastNode::new(id, NodeType::Boolean { value: value });
}

fn build_break(id: GastID) -> GastNode {
    return GastNode::new(id, NodeType::Break { });
}

fn build_continue(id: GastID) -> GastNode {
    return GastNode::new(id, NodeType::Continue { });
}

fn build_nil(id: GastID) -> GastNode {
    return GastNode::new(id, NodeType::Nil {});
}

fn build_if(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_test = obj.get("test").unwrap();
    let test = Box::new(build(json_test));

    let json_body = obj.get("body").unwrap();
    let body = Box::new(build(json_body));

    let json_orelse = obj.get("orElse").unwrap();
    let or_else = Box::new(build(json_orelse));

    return GastNode::new(id,
                         NodeType::If {
                             test: test,
                             body: body,
                             or_else: or_else,
                         });
}

fn build_while(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_test = obj.get("test").unwrap();
    let test = Box::new(build(json_test));

    let json_body = obj.get("body").unwrap();
    let body = Box::new(build(json_body));


    return GastNode::new(id,
                         NodeType::While {
                             test: test,
                             body: body,
                         });
}

fn build_block(id: GastID, node: &Json) -> GastNode {
    let array = node.as_array().unwrap();
    let mut content = Vec::new();

    for element in array {
        content.push(build(element));
    }

    return GastNode::new(id, NodeType::Block { content: content });
}

fn build_assign(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let json_targets = obj.get("targets").unwrap().as_array().unwrap();
    let mut targets = Vec::new();

    for target in json_targets {
        targets.push(build(target));
    }

    let json_value = obj.get("value").unwrap();
    let value = Box::new(build(json_value));

    return GastNode::new(id,
                         NodeType::Assignment {
                             targets: targets,
                             value: value,
                         });
}

fn build_identifier(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let name = obj.get("name").unwrap().as_string().unwrap().to_owned();
    return GastNode::new(id, NodeType::Identifier { name: name });
}

fn build_int(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let value = obj.get("value").unwrap().as_i64().unwrap();
    return GastNode::new(id, NodeType::Int { value: value });
}

fn build_float(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let value = obj.get("value").unwrap().as_f64().unwrap();
    return GastNode::new(id, NodeType::Float { value: value });
}

fn build_string(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let value = obj.get("value").unwrap().as_string().unwrap().to_owned();
    return GastNode::new(id, NodeType::String { value: value });
}

fn build_attribute(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let raw_parent = obj.get("of").unwrap();
    let attribute = obj.get("attribute").unwrap().as_string().unwrap().to_owned();
    let parent = Box::new(build(raw_parent));
    return GastNode::new(id,
                         NodeType::Attribute {
                             parent: parent,
                             attribute: attribute,
                         });
}

fn build_list(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let array = obj.get("content").unwrap().as_array().unwrap();
    let mut content = Vec::new();

    for element in array {
        content.push(build(element));
    }

    return GastNode::new(id, NodeType::List { content: content });
}

fn build_set(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let array = obj.get("content").unwrap().as_array().unwrap();
    let mut content = Vec::new();

    for element in array {
        content.push(build(element));
    }

    return GastNode::new(id, NodeType::Set { content: content });
}

fn build_sequence(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let array = obj.get("content").unwrap().as_array().unwrap();
    let mut content = Vec::new();

    for element in array {
        content.push(build(element));
    }

    return GastNode::new(id, NodeType::Sequence { content: content });
}
