use super::Pointer;
use super::OptionalMapping;
use super::Path;
use super::Collection;
use super::Representant;
use super::Scope;
use super::Mapping;
use super::GastID;
use super::CollectionBranch;
use super::CollectionChunk;

/// objects
// Object is composed of several properties it may or may not have
pub struct Object {
    is_type: bool,

    // todo remove, deprecated
    parent: Option<Pointer>, // for method objects

    extensions: Vec<Pointer>, // python-style inheritance

    elements: Collection, // for ... in x -- and x[i]
    attributes: Scope, // x.attrbitue

}

impl Object {
    pub fn new() -> Object {
        Object {
            is_type: false,
            extensions: Vec::new(),
            parent: None,
            elements: Collection::new(),
            attributes: Scope::new(),
        }
    }

    pub fn make_type(&mut self, new: bool) {
        self.is_type = new;
    }

    pub fn is_type(&self) -> bool {
        return self.is_type;
    }

    pub fn get_parent(&self) -> &Option<Pointer> {
        return &self.parent;
    }

    pub fn set_parent(&mut self, parent: Pointer) {
        self.parent = Some(parent);
    }

    pub fn extend(&mut self, tpe: Pointer) {
        self.extensions.push(tpe);
    }

    pub fn get_extension(&self) -> &Vec<Pointer> {
        return &self.extensions;
    }

    pub fn get_extension_mut(&mut self) -> &mut Vec<Pointer> {
        return &mut self.extensions;
    }

    pub fn change_branch(&mut self) {
        if self.attributes.num_frames() <= self.elements.num_frames() {
            self.elements.change_branch()
        } 
        
        if self.elements.num_frames() <= self.attributes.num_frames() {
            self.attributes.change_branch()
        } 
    }

    pub fn merge_until(&mut self, cutoff: Option<GastID>) {
        if self.attributes.num_frames() <= self.elements.num_frames() {
            self.elements.merge_until(cutoff)
        } 
        
        if self.elements.num_frames() <= self.attributes.num_frames() {
            self.attributes.merge_until(cutoff)
        } 
    }

    pub fn lift_branches(&mut self) {
        if self.attributes.num_frames() <= self.elements.num_frames() {
            self.elements.lift_branches()
        } 
        
        if self.elements.num_frames() <= self.attributes.num_frames() {
            self.attributes.lift_branches()
        } 
    }

    // attributes
    pub fn assign_attribute(&mut self, name: String, path: Path, mapping: Mapping) {
        self.attributes.set_mapping(name, path, mapping);
    }

    pub fn get_attribute(&self, name: &String) -> &OptionalMapping {
        return self.attributes.resolve_optional_identifier(name);
    }

    // elements
    pub fn size_range(&self) -> Vec<(Path, Option<usize>, Option<usize>)> {
        self.elements.size_range()
    }

    pub fn is_reliable_collection(&self) -> Vec<(Path, bool)> {
        self.elements.is_reliable()
    }

    pub fn get_element(&self, n: i16, node: &GastID) -> Mapping {
        self.elements.get_element(n, node)
    }

    pub fn get_any_element(&self, node: &GastID) -> Mapping {
        self.elements.get_any_element(node)
    }

    pub fn get_first_n_elements(&self, n: i16, node: &GastID) -> Vec<Mapping> {
        self.elements.get_first_n(n, node)
    }

    pub fn get_last_n_elements(&self, n: i16, node: &GastID) -> Vec<Mapping> {
        self.elements.get_last_n(n, node)
    }

    pub fn slice_elements(&self, start: i16, end: i16) -> Vec<(Path, CollectionBranch)> {
        self.elements.slice(start, end)
    }

    pub fn insert_element(&mut self, element: CollectionChunk, path: Path) {
        self.elements.insert(element, path)
    }

    pub fn define_elements(&mut self, content: Vec<CollectionChunk>, path: Path) {
        self.elements.define(content, path);
    }

    pub fn append_element(&mut self, element: CollectionChunk, path: Path) {
        self.elements.append(element, path)
    }

    pub fn prepend_element(&mut self, element: CollectionChunk, path: Path) {
        self.elements.prepend(element, path)
    }

    pub fn set_elements(&mut self, content: Vec<(Path, CollectionBranch)>) {
        self.elements.set_content(content)
    }
}
