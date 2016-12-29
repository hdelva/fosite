use super::Message;
use super::CHANNEL;
use std::thread::*;

pub struct Worker {
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new() -> Worker {
        let thread = {
            spawn(move || for message in CHANNEL.iter() {
                match message {
                    Message::Notification { ref source, ref content } => {
                        println!("Message from {}: {}", source, content)
                    }
                    Message::Terminate => break,
                }
            })
        };

        let worker = Worker {
            thread: thread,
        };

        return worker;
    }

    pub fn finalize(self) -> Result<()> {
    	&CHANNEL.publish({
            Message::Terminate
        });
        return self.thread.join();
    }
}
