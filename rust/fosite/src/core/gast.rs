pub type GastID = i16;

#[derive(Debug)]
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
}

#[derive(Debug)]
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
    Number { value: i64 },
    String { value: String },
    List { content: Vec<GastNode> },
    Sequence { content: Vec<GastNode> },
}