use super::GastID;
use std::collections::btree_set::Iter;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::collections::btree_set::IntoIter;

pub type PathID = Vec<GastID>;


#[derive(Clone, Debug)]
pub enum PathNode {
    Condition(PathID, i16, i16),
    Assignment(PathID, String),
    Loop(PathID),
    Return(PathID),
    Frame(PathID, Option<String>, i16, i16),
    Element(PathID, i16, i16), // element x out of y elements
}

// Element nodes are the only ones were the primary index is relevant
// the others are entirely determined by the PathID 
// we can't have executed different branches of the same conditional after all
impl Ord for PathNode {
    fn cmp(&self, other: &PathNode) -> Ordering {
        if self.is_assign() && !other.is_assign() {
            if self.get_location().len() <= other.get_location().len() {
                let len = self.get_location().len();
                if self.get_location()[0..len] == other.get_location()[0..len] {
                    return Ordering::Greater;
                }
            }
        } else if other.is_assign() && !self.is_assign() {
            if other.get_location().len() <= self.get_location().len() {
                let len = other.get_location().len();
                if other.get_location()[0..len] == self.get_location()[0..len] {
                    return Ordering::Less;
                }
            }
        }

        self.get_location().cmp(&other.get_location())
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &PathNode) -> Option<Ordering> {
        match (self, other) {
            (&PathNode::Element(ref l1, ref i1, _), &PathNode::Element(ref l2, ref i2, _)) => {
               (l1, i1).partial_cmp( &(l2, i2) )
            },
            _ => self.get_location().partial_cmp(&other.get_location())
        }
    }
}

impl PartialEq for PathNode {
    fn eq(&self, other: &PathNode) -> bool {
        match (self, other) {
            (&PathNode::Element(ref l1, ref i1, _), &PathNode::Element(ref l2, ref i2, _)) => {
               (l1, i1) == (l2, i2)
            },
            _ => self.get_location() == other.get_location(),
        }
    }
}

impl Eq for PathNode {}

impl Hash for PathNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            &PathNode::Element(ref l, ref i, _) => {
                l.hash(state);
                i.hash(state);
            },
            _ => self.get_location().hash(state),
        }
    }
}

impl PathNode {
    pub fn is_assign(&self) -> bool {
        match self {
            &PathNode::Assignment(_, _) => true,
            _ => false,
        }
    }

    pub fn is_branch(&self) -> bool {
        match self {
            &PathNode::Condition(_, _, _) => true,
            _ => false,
        }
    }

    pub fn reverse(&self) -> Vec<PathNode> {
        match self {
            &PathNode::Condition(ref l, ref x, ref y) => {
                let mut v = Vec::new();
                for i in 0..*y{
                    if i != *x {
                        v.push(PathNode::Condition(l.clone(), i, y.clone()));
                    }
                }
                return v;
            }
            _ => return vec!(self.clone()),
        }
    }

    pub fn get_location(&self) -> &PathID {
        match self {
            &PathNode::Condition(ref location, _, _) => location,
            &PathNode::Assignment(ref location, _) => location,
            &PathNode::Loop(ref location) => location,
            &PathNode::Return(ref location) => location,
            &PathNode::Frame(ref location, _, _, _) => location,
            &PathNode::Element(ref location, _, _) => location,
        }
    }

    fn mergeable(&self, other: &PathNode) -> bool {
        match (self, other) {
            (&PathNode::Condition(ref l1, ref n1, _), &PathNode::Condition(ref l2, ref n2, _)) => {
                return l1 != l2 || n1 == n2;
            }
            _ => true, // other kinds of nodes can't contradict each other
        }
    }

    // not the built-in eq() function
    // this is used to check whether or a path is contained in another
    fn equals(&self, other: &PathNode) -> bool {
        match (self, other) {
            (&PathNode::Condition(ref l1, ref i1, _), &PathNode::Condition(ref l2, ref i2, _)) => {
                return l1 == l2 && i1 == i2;
            }
            (&PathNode::Loop(ref l1), &PathNode::Loop(ref l2)) => {
                return l1 == l2;
            }
            (&PathNode::Frame(ref l1, _, ref i1, _), &PathNode::Frame(ref l2, _, ref i2, _)) => {
                return l1 == l2 && i1 == i2;
            }
            (&PathNode::Return(ref l1), &PathNode::Return(ref l2)) => {
                return l1 == l2;
            }
            (&PathNode::Assignment(ref l1, ..), &PathNode::Assignment(ref l2, ..)) => {
                return l1 == l2;
            }
            (&PathNode::Element(ref l1, ref i1, _), &PathNode::Element(ref l2, ref i2, _)) => {
                return l1 == l2 && i1 == i2;
            }
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Path {
    nodes: BTreeSet<PathNode>,
}

impl Path {
    pub fn empty() -> Self {
        Path { nodes: BTreeSet::new() }
    }

    pub fn len(&self) -> usize {
        return self.nodes.len();
    }

    pub fn get_nodes(&self) -> &BTreeSet<PathNode> {
        return &self.nodes;
    }

    pub fn iter(&self) -> Iter<PathNode> {
        return self.nodes.iter();
    }

    pub fn into_iter(self) -> IntoIter<PathNode> {
        return self.nodes.into_iter();
    }

    pub fn merge_into(&mut self, other: Path) {
        for node in other.into_iter() {
            self.nodes.insert(node);
        }
    }

    pub fn mergeable(&self, other: &Path) -> bool {
        for node in other.get_nodes() {
            let original_opt = self.nodes.get(node);
            if let Some(original) = original_opt {
                if !original.mergeable(node) {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn contains(&self, other: &Path) -> bool {
        for node in other.get_nodes() {
            let original_opt = self.nodes.get(node);
            if let Some(original) = original_opt {
                if !original.equals(node) {
                    return false;
                } 
            } else {
                return false;
            }
        }

        return true;
    }


    pub fn add_node(&mut self, element: PathNode) {
        self.nodes.insert(element);
    }

    pub fn reverse(&self) -> Vec<Path> {
        let mut result = Vec::new();
        let mut current = Path::empty();

        for node in self.nodes.iter() {
            if node.is_branch() {
                for rev in node.reverse().into_iter() {
                    let mut temp = current.clone();
                    temp.add_node(rev);
                    result.push(temp);
                }
            }

            current.add_node(node.clone());
        }
        
        return result;
    } 

    pub fn prune(&self, cutoff: &PathID) -> Path {
        let mut new = Path::empty();
        for node in self.nodes.iter() {
            if node.get_location() > cutoff {
                new.add_node(node.clone());
            }
        }
        return new;
    }
}
