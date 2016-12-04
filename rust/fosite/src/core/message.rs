use super::GastID;

#[derive(Clone, Debug)]
pub enum Message {
    Notification { source: GastID, content: String },
    Terminate,
}
