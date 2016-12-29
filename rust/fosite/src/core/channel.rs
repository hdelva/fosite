use std::sync::mpsc::{channel, Receiver, Sender};
use super::Message;
use std::sync::Mutex;

pub struct Channel {
	tx: Mutex<Sender<Message>>,
	rx: Receiver<Message>,
}

impl Channel {
	pub fn new() -> Channel {
		let (tx, rx) = channel();
		Channel {
			rx: rx,
			tx: Mutex::new(tx),
		}
	}
	
	pub fn publish(&self, item: Message) {
		let _ = self.tx.lock().unwrap().send(item);
	}
	
	pub fn iter(&self) -> ChannelIterator {
		return ChannelIterator { source: &self.rx }
	}
}

pub struct ChannelIterator<'a> {
	source: &'a Receiver<Message>
}

impl<'a> Iterator for ChannelIterator<'a> {
	type Item = Message;
	
	fn next(&mut self) -> Option<Message> {
		return self.source.recv().ok()
	}
}

unsafe impl Sync for Channel {
	
}

lazy_static! {
    pub static ref CHANNEL: Channel = Channel::new();
}

