use super::GastID;
use std::collections::btree_set::Iter;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub enum PathNode {
    Condition(GastID, bool),
    Assignment(GastID, String),
    Loop(GastID, bool),
    Return(GastID),
    Frame(GastID, Option<String>, Box<Path>),
}

impl Ord for PathNode {
    fn cmp(&self, other: &PathNode) -> Ordering {
        self.get_location().cmp(&other.get_location())
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &PathNode) -> Option<Ordering> {
        self.get_location().partial_cmp(&other.get_location())
    }
}

impl PartialEq for PathNode {
    fn eq(&self, other: &PathNode) -> bool {
        self.get_location() == other.get_location()
    }
}

impl Eq for PathNode {}

impl Hash for PathNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_location().hash(state);
    }
}

impl PathNode {
    pub fn is_branch(&self) -> bool {
        match self {
            &PathNode::Condition(_, _) => true,
            &PathNode::Assignment(_, _) => false,
            &PathNode::Loop(_, _) => true,
            &PathNode::Return(_) => false,
            &PathNode::Frame(_, _, _) => false,
        }
    }

    pub fn reverse(&self) -> Vec<PathNode> {
        match self {
            &PathNode::Condition(l, b) => vec!(PathNode::Condition(l, !b)),
            &PathNode::Assignment(l, ref t) => vec!(PathNode::Assignment(l, t.clone())),
            &PathNode::Loop(l, b) => vec!(PathNode::Loop(l, !b)),
            &PathNode::Return(l) => vec!(PathNode::Return(l)),
            &PathNode::Frame(l, ref t, ref c) => {
                let mut result = Vec::new();
                for r in c.reverse() {
                    result.push(PathNode::Frame(l, t.clone(), Box::new(r)));
                }
                return result;
            }
        }
    }

    pub fn get_location(&self) -> GastID {
        match self {
            &PathNode::Condition(location, _) => location,
            &PathNode::Assignment(location, _) => location,
            &PathNode::Loop(location, _) => location,
            &PathNode::Return(location) => location,
            &PathNode::Frame(location, _, _) => location,
        }
    }

    fn merge_into(&mut self, other: &PathNode) {
        match (self, other) {
            (&mut PathNode::Frame(_, _, ref mut n1), &PathNode::Frame(_, _, ref n2)) => {
                    n1.merge_into(n2.as_ref().clone());
            }
            _ => (),
        }
    }

    fn mergeable(&self, other: &PathNode) -> bool {
        match (self, other) {
            (&PathNode::Condition(l1, n1), &PathNode::Condition(l2, n2)) => {
                return l1 != l2 || n1 == n2;
            }
            (&PathNode::Loop(l1, t1), &PathNode::Loop(l2, t2)) => {
                return l1 != l2 || t1 == t2;
            }
            (&PathNode::Frame(_, _, ref n1), &PathNode::Frame(_, _, ref n2)) => {
                return n1.mergeable(n2);
            }
            _ => true, // other kinds of nodes can't contradict each other
        }
    }

    // not the built-in eq() function
    // this is used to check whether or a path is contained in another
    fn equals(&self, other: &PathNode) -> bool {
        match (self, other) {
            (&PathNode::Condition(l1, n1), &PathNode::Condition(l2, n2)) => {
                return l1 == l2 && n1 == n2;
            }
            (&PathNode::Loop(l1, t1), &PathNode::Loop(l2, t2)) => {
                return l1 == l2 && t1 == t2;
            }
            (&PathNode::Frame(_, _, ref n1), &PathNode::Frame(_, _, ref n2)) => {
                return n1.mergeable(n2);
            }
            (&PathNode::Return(l1), &PathNode::Return(l2)) => {
                return l1 == l2;
            }
            (&PathNode::Assignment(l1, ..), &PathNode::Assignment(l2, ..)) => {
                return l1 == l2;
            }
            _ => false,
        }
    }

    fn add_node(&mut self, other: PathNode) {
        match self {
            &mut PathNode::Frame(_, _, ref mut nodes) => {
                nodes.add_node(other);
            }
            _ => unreachable!("Trying to add to something that isn't a Frame node"),
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

    pub fn merge_into(&mut self, other: Path) {
        for node in other.get_nodes() {
            let new;

            {
                let original_opt = self.nodes.get(node);
                if let Some(original) = original_opt {
                    let mut original = original.clone();
                    original.merge_into(node);
                    new = original
                } else {
                    new = node.clone();
                }
            }

            self.nodes.insert(new);
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

    pub fn prune(&self, cutoff: GastID) -> Path {
        let mut new = Path::empty();
        for node in self.nodes.iter() {
            if node.get_location() > cutoff {
                new.add_node(node.clone());
            }
        }
        return new;
    }
}
