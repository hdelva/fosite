use super::Message;
use carboxyl::{Sink, Stream};
use std::thread::*;

pub struct Worker {
    sink: Sink<Message>,
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new(sink: Sink<Message>) -> Worker {
        let thread = {
            let sink_clone = sink.clone();
            spawn(move || {
                Worker::process(sink_clone.stream());
            })
        };

        let worker = Worker {
            sink: sink,
            thread: thread,
        };

        return worker;
    }

    fn process(stream: Stream<Message>) {
        for message in stream.events() {
            match message {
                Message::Notification { ref source, ref content } => {
                    println!("Message from {}: {}", source, content)
                }
                Message::Terminate => break,
            }
        }
    }

    pub fn finalize(self) -> Result<()> {
        self.sink.send({
            Message::Terminate
        });
        return self.thread.join();
    }
}
