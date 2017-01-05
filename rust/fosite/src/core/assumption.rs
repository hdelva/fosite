use super::GastID;
use std::collections::btree_set::Iter;

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Assumption {
    content: BTreeSet<(GastID, bool)>,
}

impl Assumption {
    pub fn empty() -> Assumption {
        Assumption { content: BTreeSet::new() }
    }

    pub fn simple(source: GastID, positive: bool) -> Assumption {
        let mut content = BTreeSet::new();
        content.insert( (source, positive));
        Assumption { content: content }
    }

    pub fn from_vec(vec: Vec<(GastID, bool)>) -> Assumption {
        let mut content = BTreeSet::new();
        for element in vec {
            content.insert( element );
        }
        Assumption { content: content }
    }

	pub fn add_element(&mut self, element: (GastID, bool)) {
		self.content.insert(element);
	}

    pub fn contains(&self, element: &(GastID, bool)) -> bool {
        return self.content.contains(element);
    }

    pub fn contains_complement(&self, element: &(GastID, bool)) -> bool {
        let &(source, positive) = element;
        let other = (source, !positive);
        return self.content.contains(&other);
    }

    pub fn add(&mut self, source: GastID, positive: bool) {
        self.content.insert((source, positive));
    }

    pub fn iter(&self) -> Iter<(GastID, bool)> {
        return self.content.iter();
    }

    pub fn get(&self) -> &BTreeSet<(GastID, bool)> {
        return &self.content;
    }

    pub fn len(&self) -> usize {
        return self.content.len().clone();
    }

    // pub fn merge<'a>(&'a self, other: &'a Assumption) -> Option<Assumption> {
    // if self.conflicts_with(other) {
    // return None
    // }
    //
    // let mut new_assumption = Assumption::empty();
    //
    // for &(source, negated) in other.get() {
    // if self.content.contains( &(source, !negated) ) {
    // conflicting assumptions
    // return None
    // } else if self.content.contains( &(source, negated) ) {
    // continue;
    // }
    //
    // new_assumption.add(source.clone(), negated.clone());
    // }
    //
    // for &(source, negated) in self.get() {
    // new_assumption.add(source.clone(), negated.clone());
    // }
    //
    // return Some(new_assumption)
    // }
    //
    // pub fn conflicts_with<'a>(&'a self, other: &'a Assumption) -> bool {
    // for &(source, negated) in other.get() {
    // if self.content.contains( &(source, !negated) ) {
    // return true
    // }
    // }
    //
    // return false
    // }
    //
    // pub fn contained_in<'a>(&'a self, other: &'a Assumption) -> bool {
    // for t in self.get() {
    // if !other.get().contains(t) {
    // return false
    // }
    // }
    //
    // return true
    // }
    //
    // pub fn opposites(original: &Assumption) -> Vec<Assumption> {
    // let mut current = vec!();
    // let mut result = vec!();
    //
    // for &(source, negated) in original.get().iter() {
    // current.push( (source.clone(), !negated) );
    // result.push(Assumption::from_vec(&current));
    // current.pop();
    // current.push( (source.clone(), negated.clone()) );
    // }
    //
    // return result
    // }
}
