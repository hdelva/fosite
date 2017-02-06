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
    Frame(GastID, Option<String>, BTreeSet<PathNode>),
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
                for node in n2 {
                    let new;

                    {
                        let original_opt = n1.get(&node);
                        if let Some(original) = original_opt {
                            let mut original = original.clone();
                            original.merge_into(node);
                            new = original
                        } else {
                            new = node.clone();
                        }
                    }

                    n1.insert(new);
                }
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
                for node in n2 {
                    let original_opt = n1.get(node);
                    if let Some(original) = original_opt {
                        if !original.mergeable(node) {
                            return false;
                        }
                    }
                }
                return true;
            }
            _ => true, // other kinds of nodes can't contradict each other
        }
    }

    fn add_node(&mut self, other: PathNode) {
        match self {
            &mut PathNode::Frame(_, _, ref mut nodes) => {
                nodes.insert(other);
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

    pub fn add_node(&mut self, element: PathNode) {
        self.nodes.insert(element);
    }
}
