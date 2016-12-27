use super::Message;
use carboxyl::{Sink};
use std::thread::*;

pub struct Worker {
    sink: Sink<Message>,
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new(sink: Sink<Message>) -> Worker {
        let thread = {
            let stream = sink.stream();
            let events = stream.events();
            
            spawn(move || for message in events {
	            match message {
	                Message::Notification { ref source, ref content } => {
	                    println!("Message from {}: {}", source, content)
	                },
	                Message::Terminate => {
	                	break
	                },
	            }
	        })
        };

        let worker = Worker {
            sink: sink,
            thread: thread,
        };

        return worker;
    }

    pub fn finalize(self) -> Result<()> {
        self.sink.send({
            Message::Terminate
        });
        return self.thread.join();
    }
}
