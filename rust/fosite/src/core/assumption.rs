use super::GastID;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Assumption {
	content: Vec<(GastID, bool)>,
}

impl Assumption {
	pub fn empty() -> Assumption {
		Assumption {
			content: Vec::new(),
		}
	}
	
	pub fn from_vec(content: &Vec< (GastID, bool) >) -> Assumption {
		Assumption {
			content: content.clone(),
		}
	}
	
	pub fn add(&mut self, source: GastID, negated: bool) {
		self.content.push( (source, negated) );
	}
	
	pub fn get(&self) -> &Vec<(GastID, bool)> {
		return &self.content
	}
	
	pub fn merge<'a>(&'a self, other: &'a Assumption) -> Option<Assumption> {
		if self.conflicts_with(other) {
			return None
		}
		
		let mut new_assumption = Assumption::empty();
		
		for &(source, negated) in other.get() {
			if self.content.contains( &(source, !negated) ) {
				// conflicting assumptions
				return None
			} else if self.content.contains( &(source, negated) ) {
				continue;
			}
			
			new_assumption.add(source.clone(), negated.clone());
		}
		
		for &(source, negated) in self.get() {
			new_assumption.add(source.clone(), negated.clone());
		}
		
		return Some(new_assumption)
    }
	
	pub fn conflicts_with<'a>(&'a self, other: &'a Assumption) -> bool {
		for &(source, negated) in other.get() {
			if self.content.contains( &(source, !negated) ) {
				return true
			}
		}
		
		return false
	}
	
	pub fn contained_in<'a>(&'a self, other: &'a Assumption) -> bool {
		for t in self.get() {
			if !other.get().contains(t) {
				return false
			}
		}
		
		return true
	}

	pub fn opposites(original: &Assumption) -> Vec<Assumption> {
		let mut current = vec!();
		let mut result = vec!();
		
		for &(source, negated) in original.get().iter() {
			current.push( (source.clone(), !negated) );
			result.push(Assumption::from_vec(&current));
			current.pop();
			current.push( (source.clone(), negated.clone()) );
		}
		
		return result
	}
}