use super::AnalysisItem;
use super::Mapping;

use std::collections::HashMap;

pub struct Watch {
    pub before: HashMap<AnalysisItem, Mapping>,
    pub after: HashMap<AnalysisItem, Mapping>,
    in_setup: bool,
}

impl Watch {
    pub fn new() -> Self {
        Watch {
            before: HashMap::new(),
            after: HashMap::new(),
            in_setup: true,
        }
    }

    pub fn toggle(&mut self) {
        self.in_setup = !self.in_setup;
    }

    pub fn store(&mut self, identifier: AnalysisItem, mapping: Mapping) {
        if self.in_setup {
            self.before.insert(identifier.clone(), mapping.clone());
            self.after.insert(identifier, mapping);
        } else {
            if self.before.contains_key(&identifier) {
                self.after.insert(identifier, mapping);
            }
        }
    }
}

