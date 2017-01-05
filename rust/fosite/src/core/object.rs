use std::collections::HashMap;

use super::Pointer;
use super::OptionalMapping;
use super::Assumption;
use super::Collection;
use super::Representant;
use super::Scope;
use super::Mapping;

/// objects
// Object is composed of several properties it may or may not have
pub struct Object {
    type_property: bool,

    // todo remove, deprecated
    parent_property: Option<Pointer>, // for method objects

    // todo, keep optional?
    collection_property: Option<CollectionProperty>, // for ... in x

    composite_property: CompositeProperty, // x.attrbitue
    extension_property: Vec<Pointer>, // python-style inheritance
}

impl Object {
    pub fn new() -> Object {
        Object {
            type_property: false,
            extension_property: Vec::new(),
            parent_property: None,
            collection_property: None,
            composite_property: CompositeProperty::new(),
        }
    }

    pub fn set_type(&mut self, new: bool) {
        self.type_property = new;
    }

    pub fn is_type(&self) -> bool {
        return self.type_property;
    }

    pub fn get_parent(&self) -> &Option<Pointer> {
        return &self.parent_property;
    }

    pub fn set_parent(&mut self, parent: Pointer) {
        self.parent_property = Some(parent);
    }

    pub fn extends(&mut self, tpe: Pointer) {
        self.extension_property.push(tpe);
    }

    pub fn get_extension(&self) -> &Vec<Pointer> {
        return &self.extension_property;
    }

    pub fn get_extension_mut(&mut self) -> &mut Vec<Pointer> {
        return &mut self.extension_property;
    }

    pub fn iterate(&self) -> Option<Pointer> {
        // todo
        return None;
    }

    pub fn make_collection(&mut self) {
        self.collection_property = Some(CollectionProperty::new());
    }

    pub fn assign_attribute(&mut self, name: String, assumption: Assumption, mapping: Mapping) {
        self.composite_property.assign_attribute(name, assumption, mapping);
    }

    pub fn get_attribute(&self, name: &String) -> &OptionalMapping {
        return self.composite_property.get_attribute(name);
    }
    
    pub fn change_branch(&mut self) {
    	self.composite_property.change_branch();
    }
    
    pub fn merge_branches(&mut self) {
    	self.composite_property.merge_branches();
    }
}

/// Object Properties

struct CollectionProperty {
    collections: HashMap<Assumption, Collection>,
}

impl CollectionProperty {
    fn new() -> CollectionProperty {
        CollectionProperty { collections: HashMap::new() }
    }

    fn define(&mut self, content: Vec<Representant>) {
        let mut collection = Collection::empty();
        collection.define(content);
        self.collections.insert(Assumption::empty(), collection);
    }

    // fn expand_collections_(&mut self, assumption: &Assumption) {
    // let mut new_collections = HashMap::new();
    //
    // for opposite in Assumption::opposites(assumption) {
    // for (old_assumption, collection) in self.collections.iter() {
    // let optional_assumption = old_assumption.merge(&opposite);
    //
    // match optional_assumption {
    // Some(new_assumption) => {
    // new_collections.insert(new_assumption, collection.clone());
    // },
    // None => {
    // new_collections.insert(old_assumption.clone(), collection.clone());
    // }
    // }
    // }
    // }
    //
    // self.collections = new_collections;
    // }
    //
}

struct CompositeProperty {
    // todo split in private/public/(protected)
    namespace: Scope,
}

impl CompositeProperty {
    fn new() -> CompositeProperty {
        CompositeProperty { namespace: Scope::new() }
    }

    fn assign_attribute(&mut self, name: String, assumption: Assumption, mapping: Mapping) {
        self.namespace.set_mapping(name, assumption, mapping);
    }

    fn get_attribute(&self, name: &String) -> &OptionalMapping {
        self.namespace.resolve_optional_identifier(name)
    }
    
    fn change_branch(&mut self) {
    	self.namespace.change_branch();
    }
    
    fn merge_branches(&mut self) {
    	self.namespace.merge_branches();
    }
}
