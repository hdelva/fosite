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
    Sequence { content: Vec<GastNode> },
    Block { content: Vec<GastNode> },
    If { test: Box<GastNode>,
    	 body: Box<GastNode>,
    	 or_else: Box<GastNode>,
    },
    BinOp { left: Box<GastNode>,
            right: Box<GastNode>,
            op: String,
            associative: bool,
    }
}

impl NodeType {
    fn to_string(&self) -> String {
        match self {
            &NodeType::Identifier {ref name} => name.clone(),
            &NodeType::Attribute {ref parent, ref attribute} => format!("{}.{}", parent.kind.to_string(), attribute),
            _ => panic!("Node {:?} doesn't have a string representation", self),
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
        "binop" => build_binop(id, node),
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

    return GastNode::new(id, NodeType::BinOp {
        left: left,
        right: right,
        op: op,
        associative: ass,
    })
}

fn build_if(id: GastID, node: &Json) -> GastNode {
	let obj = node.as_object().unwrap();
	
	let json_test = obj.get("test").unwrap();
    let test = Box::new(build(json_test));
    
    let json_body = obj.get("body").unwrap();
    let body = Box::new(build(json_body));
    
    let json_orelse = obj.get("orElse").unwrap();
    let or_else = Box::new(build(json_orelse));
    
    return GastNode::new(id, NodeType::If {test: test, body: body, or_else: or_else});
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

fn build_sequence(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let array = obj.get("content").unwrap().as_array().unwrap();
    let mut content = Vec::new();

    for element in array {
        content.push(build(element));
    }

    return GastNode::new(id, NodeType::Sequence { content: content });
}
