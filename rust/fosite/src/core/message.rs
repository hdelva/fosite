use super::GastID;
use super::Assumption;

#[derive(Clone, Debug)]
pub enum Message {
	Error {source: GastID, assumption: Assumption, content: String },
	Warning {source: GastID, assumption: Assumption, content: String },
	Input { source: GastID, line: i16, col: i16 },
    Notification { source: GastID, content: String },
    Terminate,
}
