use super::GastID;
use super::Path;
use super::message::*;
use super::CHANNEL;

use std::collections::HashMap;

pub struct Signaler {

}

impl Signaler {
    pub fn new() -> Signaler {
        Signaler { }
    }

    pub fn out_of_bounds(&self, source: GastID, target: String, paths: Vec<(Path, i16)>) {
        let mut items = HashMap::new();

        items.insert("target".to_owned(), MessageItem::String(target));

        for (index, (path, max)) in paths.into_iter().enumerate() {
            items.insert(format!("path {}", index), MessageItem::Path(path));
            items.insert(format!("path {} max", index), MessageItem::Number(max));
        }

        let message = Message::Warning {
            source: source,
            kind: WINDEX_BOUNDS,
            content: items,
        };
        &CHANNEL.publish(message);
    }
}

lazy_static! {
    pub static ref SIGNALER: Signaler = Signaler::new();
}
