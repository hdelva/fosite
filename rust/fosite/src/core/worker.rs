use super::Message;
use super::CHANNEL;
use std::thread::*;
use std::collections::HashMap;
use super::GastID;

pub struct Worker {
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new() -> Worker {
    	let mut logger = Logger::new();
    	
        let thread = {
            spawn(move || 
            	logger.message_loop()
            )
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


struct Logger {
	sources: HashMap<GastID, (i16, i16)>,
}

impl Logger {
	fn new() -> Logger {
		Logger {
			sources: HashMap::new(),
		}
	}
	
	fn message_loop(&mut self) {
		for message in CHANNEL.iter() {
            match message {
            	Message::Error { ref source, ref assumption, ref content } => {
                    println!("Error from node {} at {:?}\n  under the following assumptions:\n    {:?} \n  {:?}\n", source, self.sources.get(source), assumption, content)
                },
            	Message::Warning { ref source, ref assumption, ref content } => {
                    println!("Warning from node {} at {:?}\n  under the following assumptions:\n    {:?} \n  {:?}\n", source, self.sources.get(source), assumption, content)
                },
                Message::Notification { ref source, ref content } => {
                    //println!("Message from node {} at {:?}: {}", source, self.sources.get(source), content)
                },
                Message::Input {source, line, col} => {
                	//println!("mapping node {} to ({}, {})", source, line, col);
                	self.sources.insert(source, (line, col));
                },
                Message::Terminate => break,
            }
        }
	}
}
