use rustc_serialize::json::*;

use super::Message;
use super::CHANNEL;
use super::AnalysisItem;

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
    Dict { content: Vec<GastNode> },
    Pair { first: Box<GastNode>, second: Box<GastNode> },
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
    ForEach {
        before: Box<GastNode>,
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
    },
    Generator {
        source: Box<GastNode>,
        target: Box<GastNode>,
    },
    Filter {
        source: Box<GastNode>,
        condition: Box<GastNode>,
    },
    Map {
        source: Box<GastNode>,
        op: Box<GastNode>,
    },
    AndThen {
        first: Box<GastNode>,
        second: Box<GastNode>,
    },
    Call {
        target: Box<GastNode>,
        args: Vec<GastNode>,
        kwargs: Vec<GastNode>,
    },
    Import {
        module: String,
        parts: Vec<(String, String)>,
        into: Option<String>,
    },
    Negate {
        value: Box<GastNode>,
    },
    Slice {
        target: Box<GastNode>,
        lower: Box<GastNode>,
        upper: Box<GastNode>,
    },
    Argument {
        name: String,
        value: Box<GastNode>,
    },
    FunctionDef {
        name: String,
        args: Vec<GastNode>,
        kw_args: Vec<GastNode>,
        vararg: Option<String>,
        kw_vararg: Option<String>,
        body: Box<GastNode>,
    },
    Return {
        value: Box<GastNode>,
    }
}

impl NodeType {
    fn to_string(&self) -> String {
        match *self {
            NodeType::Identifier { ref name } => name.clone(),
            NodeType::Attribute { ref parent, ref attribute } => {
                format!("{}.{}", parent.kind.to_string(), attribute)
            }
            NodeType::Index { ref target, ref index } => {
                format!("{}[{}]", target.to_string(), index.to_string())
            }
            NodeType::Int {ref value} => {
                value.to_string()
            }
            NodeType::Float {ref value} => {
                value.to_string()
            }
            NodeType::String {ref value} => {
                value.clone()
            }
            NodeType::Call {ref target, ref args, ..} => {
                let pls: Vec<String> = args.iter().map(|x| x.to_string()).collect();
                // todo add kwargs
                format!("{}({})", target.to_string(), pls.join(", "))
            }
            _ => format!("Node {:?} doesn't have a string representation", self),
        }
    }

    pub fn to_analysis_item(&self) -> Option<AnalysisItem> {
        match *self {
            NodeType::Identifier { ref name } => Some(AnalysisItem::Identifier(name.clone())),
            NodeType::Attribute { ref parent, ref attribute } => {
                let parent_item = parent.as_ref().kind.to_analysis_item();
                if let Some(item) = parent_item {
                    Some(AnalysisItem::Attribute(Box::new(item), attribute.clone()))
                } else {
                    None
                }
            }  
            _ => None,
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
        "foreach" => build_foreach(id, node),
        "binop" => build_binop(id, node),
        "nil" => build_nil(id),
        "boolean" => build_bool(id, node),
        "boolop" => build_boolop(id, node),
        "break" => build_break(id),
        "continue" => build_continue(id),
        "unop" => build_unop(id, node),
        "index" => build_index(id, node),
        "set" => build_set(id, node),
        "dictionary" => build_dict(id, node),
        "pair" => build_pair(id, node),
        "generator" => build_generator(id, node),
        "filter" => build_filter(id, node),
        "map" => build_map(id, node),
        "andthen" => build_andthen(id, node),
        "call" => build_call(id, node),
        "import" => build_import(id, node),
        "negate" => build_negate(id, node),
        "slice" => build_slice(id, node),
        "argument" => build_argument(id, node),
        "function" => build_function(id, node),
        "return" => build_return(id, node),
        _ => panic!("unsupported JSON node: {:?}", node),
    };

    if let (Some(line), Some(col)) = (line, col) {
        let message = Message::Input {
            source: id,
            line: line.as_i64().unwrap() as i16,
            col: col.as_i64().unwrap() as i16,
            node: node.clone(),
        };

        CHANNEL.publish(message);
    }

    node
}

fn build_import(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_module = obj.get("module").unwrap();
    let module = json_module.as_string().unwrap().to_owned();

    let json_into = obj.get("into").unwrap();
    let into = if !json_into.is_null() {
        Some(json_into.as_string().unwrap().to_owned())
    } else {
        None
    };

    let json_parts = obj.get("parts").unwrap();
    let array = json_parts.as_array().unwrap();
    
    let mut parts = Vec::new();

    for part in array {
        let pair = part.as_array().unwrap();
        let original = pair[0].as_string().unwrap().to_owned();
        let alias = pair[1].as_string().unwrap().to_owned();
        parts.push((original, alias));
    }

    GastNode::new(id,
                         NodeType::Import {
                             module: module,
                             parts: parts,
                             into: into,
                         })
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

    GastNode::new(id,
                         NodeType::BinOp {
                             left: left,
                             right: right,
                             op: op,
                             associative: ass,
                         })
}

fn build_return(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_value = obj.get("value").unwrap();
    let value = Box::new(build(json_value));

    GastNode::new(id,
                         NodeType::Return {
                             value: value,
                         })
}

fn build_argument(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_name = obj.get("name").unwrap();
    let name = json_name.as_string().unwrap().to_owned();

    let json_value = obj.get("value").unwrap();
    let value = Box::new(build(json_value));

    GastNode::new(id,
                         NodeType::Argument {
                             name: name,
                             value: value,
                         })
}

fn build_slice(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_target = obj.get("target").unwrap();
    let target = Box::new(build(json_target));

    let json_lower = obj.get("lower").unwrap();
    let lower = Box::new(build(json_lower));

    let json_upper = obj.get("upper").unwrap();
    let upper = Box::new(build(json_upper));

    GastNode::new(id,
                         NodeType::Slice {
                             target: target,
                             lower: lower,
                             upper: upper,
                         })
}

fn build_negate(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_value = obj.get("value").unwrap();
    let value = Box::new(build(json_value));

    GastNode::new(id,
                         NodeType::Negate {
                             value: value,
                         })
}

fn build_index(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_target = obj.get("target").unwrap();
    let target = Box::new(build(json_target));

    let json_index = obj.get("index").unwrap();
    let index = Box::new(build(json_index));

    GastNode::new(id,
                         NodeType::Index {
                             target: target,
                             index: index,
                         })
}

fn build_unop(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_value = obj.get("value").unwrap();
    let value = Box::new(build(json_value));

    let json_op = obj.get("op").unwrap();
    let op = json_op.as_string().unwrap().to_owned();

    GastNode::new(id,
                         NodeType::UnOp {
                             value: value,
                             op: op,
                         })
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

    GastNode::new(id,
                         NodeType::BoolOp {
                             left: left,
                             right: right,
                             op: op,
                             reversed: reversed,
                             negated: negated,
                         })
}

fn build_bool(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_value = obj.get("value").unwrap();
    let value = json_value.as_boolean().unwrap();

    GastNode::new(id, NodeType::Boolean { value: value })
}

fn build_break(id: GastID) -> GastNode {
    GastNode::new(id, NodeType::Break { })
}

fn build_continue(id: GastID) -> GastNode {
    GastNode::new(id, NodeType::Continue { })
}

fn build_nil(id: GastID) -> GastNode {
    GastNode::new(id, NodeType::Nil {})
}

fn build_if(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_test = obj.get("test").unwrap();
    let test = Box::new(build(json_test));

    let json_body = obj.get("body").unwrap();
    let body = Box::new(build(json_body));

    let json_orelse = obj.get("orElse").unwrap();
    let or_else = Box::new(build(json_orelse));

    GastNode::new(id,
                         NodeType::If {
                             test: test,
                             body: body,
                             or_else: or_else,
                         })
}

fn build_function(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let name = obj.get("name").unwrap().as_string().unwrap().to_owned();

    let json_args = obj.get("positional_args").unwrap();
    let mut args = Vec::new();
    for node in json_args.as_array().unwrap() {
        args.push(build(node));
    }

    let json_kwargs = obj.get("keyword_args").unwrap();
    let mut kwargs = Vec::new();
    for node in json_kwargs.as_array().unwrap() {
        kwargs.push(build(node));
    }

    let json_vararg = obj.get("vararg").unwrap();
    let vararg = if json_vararg.is_null() {
        None
    } else {
        Some(json_vararg.as_string().unwrap().to_owned())
    };

    let json_kw_vararg = obj.get("kw_vararg").unwrap();
    let kw_vararg = if json_kw_vararg.is_null() {
        None
    } else {
        Some(json_kw_vararg.as_string().unwrap().to_owned())
    };

    let json_body = obj.get("body").unwrap();
    let body = Box::new(build(json_body));

    GastNode::new(id,
                         NodeType::FunctionDef {
                             name: name,
                             body: body,
                             args: args,
                             kw_args: kwargs,
                             vararg: vararg,
                             kw_vararg: kw_vararg,
                         })
}



fn build_while(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_test = obj.get("test").unwrap();
    let test = Box::new(build(json_test));

    let json_body = obj.get("body").unwrap();
    let body = Box::new(build(json_body));


    GastNode::new(id,
                         NodeType::While {
                             test: test,
                             body: body,
                         })
}

fn build_foreach(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let json_before = obj.get("before").unwrap();
    let before = Box::new(build(json_before));

    let json_body = obj.get("body").unwrap();
    let body = Box::new(build(json_body));

    GastNode::new(id,
                         NodeType::ForEach {
                             before: before,
                             body: body,
                         })
}

fn build_block(id: GastID, node: &Json) -> GastNode {
    let array = node.as_array().unwrap();
    let mut content = Vec::new();

    for element in array {
        content.push(build(element));
    }

    GastNode::new(id, NodeType::Block { content: content })
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

    GastNode::new(id,
                         NodeType::Assignment {
                             targets: targets,
                             value: value,
                         })
}

fn build_identifier(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let name = obj.get("name").unwrap().as_string().unwrap().to_owned();
    GastNode::new(id, NodeType::Identifier { name: name })
}

fn build_int(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let value = obj.get("value").unwrap().as_i64().unwrap();
    GastNode::new(id, NodeType::Int { value: value })
}

fn build_float(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let value = obj.get("value").unwrap().as_f64().unwrap();
    GastNode::new(id, NodeType::Float { value: value })
}

fn build_string(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let value = obj.get("value").unwrap().as_string().unwrap().to_owned();
    GastNode::new(id, NodeType::String { value: value })
}

fn build_attribute(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let raw_parent = obj.get("of").unwrap();
    let attribute = obj.get("attribute").unwrap().as_string().unwrap().to_owned();
    let parent = Box::new(build(raw_parent));
    GastNode::new(id,
                         NodeType::Attribute {
                             parent: parent,
                             attribute: attribute,
                         })
}

fn build_list(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let array = obj.get("content").unwrap().as_array().unwrap();
    let mut content = Vec::new();

    for element in array {
        content.push(build(element));
    }

    GastNode::new(id, NodeType::List { content: content })
}

fn build_call(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();

    let target_json = obj.get("name").unwrap();
    let target = build(target_json);

    let json_args = obj.get("positional_args").unwrap().as_array().unwrap();
    let mut args = Vec::new();

    for element in json_args {
        args.push(build(element));
    }

    let json_kwargs = obj.get("keyword_args").unwrap().as_array().unwrap();
    let mut kwargs = Vec::new();

    for element in json_kwargs {
        kwargs.push(build(element));
    }

    GastNode::new(id, NodeType::Call { target: Box::new(target), args: args, kwargs: kwargs })
}

fn build_set(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let array = obj.get("content").unwrap().as_array().unwrap();
    let mut content = Vec::new();

    for element in array {
        content.push(build(element));
    }

    GastNode::new(id, NodeType::Set { content: content })
}

fn build_dict(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let array = obj.get("content").unwrap().as_array().unwrap();
    let mut content = Vec::new();

    for element in array {
        content.push(build(element));
    }

    GastNode::new(id, NodeType::Dict { content: content })
}

fn build_pair(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    
    let json_first = obj.get("first").unwrap();
    let first = Box::new(build(json_first));

    let json_second = obj.get("second").unwrap();
    let second = Box::new(build(json_second));

    GastNode::new(id, NodeType::Pair { first: first, second: second })
}

fn build_sequence(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    let array = obj.get("content").unwrap().as_array().unwrap();
    let mut content = Vec::new();

    for element in array {
        content.push(build(element));
    }

    GastNode::new(id, NodeType::Sequence { content: content })
}

fn build_generator(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    
    let json_source = obj.get("source").unwrap();
    let source = Box::new(build(json_source));

    let json_target = obj.get("target").unwrap();
    let target = Box::new(build(json_target));

    GastNode::new(id, NodeType::Generator { source: source, target: target })
}

fn build_filter(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    
    let json_source = obj.get("source").unwrap();
    let source = Box::new(build(json_source));

    let json_condition = obj.get("condition").unwrap();
    let condition = Box::new(build(json_condition));

    GastNode::new(id, NodeType::Filter { source: source, condition: condition })
}

fn build_map(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    
    let json_source = obj.get("source").unwrap();
    let source = Box::new(build(json_source));

    let json_op = obj.get("op").unwrap();
    let op = Box::new(build(json_op));

    GastNode::new(id, NodeType::Map { source: source, op: op })
}

fn build_andthen(id: GastID, node: &Json) -> GastNode {
    let obj = node.as_object().unwrap();
    
    let json_first = obj.get("first").unwrap();
    let first = Box::new(build(json_first));

    let json_second = obj.get("second").unwrap();
    let second = Box::new(build(json_second));

    GastNode::new(id, NodeType::AndThen { first: first, second: second })
}