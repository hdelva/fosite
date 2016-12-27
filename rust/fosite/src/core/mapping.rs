use super::Pointer;
use super::Assumption;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Mapping {
	pub assumption: Assumption,
	pub address: Pointer,
}

impl Mapping {
	pub fn new(ass: Assumption, add: Pointer) -> Mapping {
		Mapping {
			assumption: ass,
			address: add,
		}
	}
}

#[derive(Debug, Clone)]
pub struct OptionalMapping {
	possibilities: HashMap<Assumption, Option<Pointer>>,
}

impl OptionalMapping {
    pub fn new() -> OptionalMapping {
    	// seed the optional mapping
		// need some root to branch from
		let mut map = HashMap::new();
		map.insert(Assumption::empty(), None);
	
        OptionalMapping {
            possibilities: map,
        }
    }
    
    pub fn get_possibilities(&self) -> &HashMap<Assumption, Option<Pointer>> {
    	return &self.possibilities
    }
    
    pub fn add_possibility(&mut self, assumption: Assumption, address: Pointer) {
    	if !self.possibilities.contains_key(&assumption) {
	    	self.expand_possibilities(&assumption);
    	}
    	
    	self.possibilities.insert(assumption, Some(address));
    }
        
    pub fn expand_possibilities(&mut self, assumption: &Assumption) {
	    let mut new_possibilities = HashMap::new();
	    
	    for opposite in Assumption::opposites(assumption) {
	    	for (old_assumption, address) in self.possibilities.iter() {
	    		let optional_assumption = old_assumption.merge(&opposite);
	    		
	    		match optional_assumption {
	    			Some(new_assumption) => {
	    				new_possibilities.insert(new_assumption, address.clone());
	    			},
	    			None => {
	    				new_possibilities.insert(old_assumption.clone(), address.clone());
	    			}
	    		}		
	    	} 
	    }
	    
	    self.possibilities = new_possibilities;
    }
}