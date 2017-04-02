use super::VirtualMachine;
use super::Pointer;

use std::collections::HashMap;

type Part = Fn(&mut VirtualMachine) -> Pointer;

pub struct Module {
    parts: HashMap<String, Box<Part>>,
}

impl Module {
    pub fn new() -> Self {
        Module {
            parts: HashMap::new(),
        }
    }

    pub fn add_part(&mut self, name: String, part: Box<Part>) {
        self.parts.insert(name, part);
    }

    pub fn make_object(&self, vm: &mut VirtualMachine, names: Vec<(String, String)>) -> Vec<(String, Pointer)> {
        let mut pointers = Vec::new();

        if names.len() == 0 {
            for (name, part) in self.parts.iter() {
                let pointer = part(vm);
                pointers.push((name.clone(), pointer));
            }
        }

        for (name, alias) in names {
            if let Some(part) = self.parts.get(&name) {
                let pointer = part(vm);
                pointers.push((alias, pointer));
            } else {
                //todo warning/error
            }
        }

        return pointers;
    }
}