#[derive(Debug)]
pub enum GastNode {
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