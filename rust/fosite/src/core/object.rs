use std::collections::HashMap;

use super::Pointer;

/// objects
// Object is composed of several properties it may or may not have
pub struct Object {
    extension_property: Option<Pointer>,            // python-style inheritance
    parent_property: Option<Pointer>,               // for method objects
    iterable_property: Option<IterableObject>,      // for ... in x
    indexable_property: Option<IndexableObject>,    // x[...]
    composite_property: Option<CompositeObject>,    // x.attrbitue
}

impl Object {
    pub fn new() -> Object {
        Object {
            extension_property: None,
            parent_property: None,
            iterable_property: None,
            indexable_property: None,
            composite_property: None,
        }
    }

    pub fn get_parent(&self) -> &Option<Pointer> {
        return &self.parent_property
    }

    pub fn set_parent(&mut self, parent: Pointer) {
        self.parent_property = Some(parent);
    }

    pub fn extends(&mut self, tpe: Pointer) {
        self.extension_property = Some(tpe);
    }

    pub fn get_extension(&self) -> &Option<Pointer> {
        return &self.extension_property
    }

    pub fn get_extension_mut(&mut self) -> &mut Option<Pointer> {
        return &mut self.extension_property
    }

    pub fn iterate(&mut self) -> Option<Pointer> {
        // limit the possible types to iterable types
        // return a reference to the object representing its kind of elements
        match self.iterable_property {
            Some(ref property) => Some(property.element.clone()),
            _ => None,
        }
    }

    pub fn index(&mut self) -> Option<Pointer> {
        // limit the possible types to indexable types
        // return a reference to the object representing its kind of elements
        match self.indexable_property {
            Some(ref property) => Some(property.element.clone()),
            _ => None,
        }
    }

    pub fn reference_attribute(&mut self, name: &String) -> Option<Pointer> {
        match self.composite_property {
            Some(ref property) => {
                match property.attributes.get(name){
                    // the referenced property was part of a previous assignment
                    // just return it
                    Some(attribute) => Some(attribute.clone()),
                    _ => None,
                }
            },
            _ => None,
        }
    }

    pub fn assign_attribute(&mut self, name: &String, value: Pointer) {
        // sets the attribute reference
        match self.composite_property {
            Some(ref mut property) => {
                property.assign_attribute(name, value);
            },
            _ => {
                let mut property = CompositeObject::new();
                property.assign_attribute(name, value);
                self.composite_property = Some(property);
            },
        }
    }
}

/// Object Properties

struct CompositeObject {
    //todo split in private/public/(protected)
    attributes: HashMap<String, Pointer>,
}

impl CompositeObject {
    fn new() -> CompositeObject {
        CompositeObject {
            attributes: HashMap::new(),
        }
    }

    fn add_attribute(&mut self, name: &String, address: Pointer)  {
        self.attributes.insert(name.clone(), address);
    }

    fn assign_attribute(&mut self, name: &String, value: Pointer) {
        self.attributes.insert(name.clone(), value);
    }
}

struct IterableObject  {
    element: Pointer,
}

impl IterableObject {
    fn new(element: Pointer) -> IterableObject {
        IterableObject {
            element: element
        }
    }
}

struct IndexableObject  {
    element: Pointer,
}

impl IndexableObject {
    fn new(element: Pointer) -> IndexableObject {
        IndexableObject {
            element: element
        }
    }
}