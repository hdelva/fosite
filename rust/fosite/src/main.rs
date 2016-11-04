#![allow(dead_code)]

use std::collections::HashSet;
use std::collections::HashMap;

/// type aliases

type Type = i16;

/// Placeholders

trait KnowledgeBase {
    // function belong to a namespace
    fn functions(&self, namespace: Namespace, name: &String) -> Vec<&CallableObject>;

    // methods belong to a type (~class)
    fn methods(&self, class: &Type, name: &String) -> Vec<&CallableObject>;

    // take ownership of the Object
    fn hold_this(&self, Object) -> &Object;

    // all known types
    fn all_types(&self) -> &HashSet<Type>;

    // all iterable types
    fn iterable_types(&self) -> &HashSet<Type>;

    // all indexable types
    fn indexable_types(&self) -> &HashSet<Type>;

    // all types with a certain attribute in its definition
    fn types_with_attribute(&self, name: &String) -> &HashSet<Type>;
}

//todo implement for each builtin function
trait BuiltinFunction {
    fn call(&self, kb: &mut KnowledgeBase, args: [&Object]);
}

trait FunctionDefinition {

}

trait Namespace {

}

/// objects

// Object is composed of several properties it may or may not have
struct Object<'a> {
    type_property: Option<TypedObject>,
    iterable_property: Option<IterableObject<'a>>,
    indexable_property: Option<IndexableObject<'a>>,
    composite_property: Option<CompositeObject<'a>>,
}

impl<'a> Object<'a> {
    fn new<'b>() -> Object<'b> {
        Object {
            type_property: None,
            iterable_property: None,
            indexable_property: None,
            composite_property: None,
        }
    }

    fn get_types(&self, kb: &'a KnowledgeBase) -> &HashSet<Type> {
        match self.type_property {
            None => return kb.all_types(),
            Some(ref property) => return property.get_types()
        }
    }

    fn limit_types(&mut self, kb: &'a KnowledgeBase, limits: &HashSet<Type>) {
        match self.type_property {
            None => {
                let all = kb.all_types();
                let possible = all.intersection(limits).cloned().collect();
                let property = TypedObject::new(possible);
                self.type_property = Some(property);
            },
            Some(ref mut property) => {
                property.limit(limits);
            }
        }
    }

    fn iterate(&mut self, kb: &'a KnowledgeBase) -> &'a Object {
        match self.iterable_property {
            None => {
                self.limit_types(kb, kb.iterable_types());

                let element = Object::new();
                let reference = kb.hold_this(element);
                let property = IterableObject::new(reference);
                self.iterable_property = Some(property);
                return reference
            },
            Some(ref property) => {
                return property.element;
            }
        }
    }

    fn index(&mut self, kb: &'a KnowledgeBase) -> &'a Object {
        match self.indexable_property {
            None => {
                self.limit_types(kb, kb.indexable_types());

                let element = Object::new();
                let reference = kb.hold_this(element);
                let property = IndexableObject::new(reference);
                self.indexable_property = Some(property);
                return reference
            },
            Some(ref property) => {
                return property.element;
            }
        }
    }

    fn reference_attribute(&mut self, kb: &'a KnowledgeBase, name: &String) -> &'a Object {
        match self.composite_property {
            None => {
                self.composite_property = Some(CompositeObject::new());
            },
            Some(ref property) => {
                match property.attributes.get(name){
                    Some(attribute) => return attribute,
                    _ => (),
                }
            },
        }

        self.limit_types(kb, kb.types_with_attribute(name));

        return self.composite_property.as_mut().unwrap().add_attribute(kb, name);
    }

    fn assign_attribute(&mut self, name: &String, value: &'a Object) {
        match self.composite_property {
            None => {
                let mut property = CompositeObject::new();
                property.assign_attribute(name, value);
                self.composite_property = Some(property);
            },
            Some(ref mut property) => {
                property.assign_attribute(name, value);
            }
        }
    }

    fn method(&mut self, kb: &'a KnowledgeBase, name: &String) -> Vec<&CallableObject> {
        //todo limit types to callables?
        
        let mut result = Vec::new();

        for t in self.get_types(kb) {
            let mut partial = kb.methods(t, name);
            result.append(&mut partial);
        }

        return result;
    }

    fn is_valid(&self) -> bool {
        match self.type_property {
            None => true,
            Some(ref property) => property.types.len() > 0,
        }
    }
}

/// Object Properties

struct TypedObject {
    types: HashSet<Type>,
}

impl TypedObject {
    fn new(types: HashSet<Type>) -> TypedObject {
        TypedObject{
            types: types,
        }
    }

    fn limit(&mut self, limits: &HashSet<Type>) {
        self.types = self.types.intersection(limits).cloned().collect();
    }

    fn get_types(&self) -> &HashSet<Type> {
        return &self.types;
    }
}

struct CompositeObject<'a> {
    attributes: HashMap<String, &'a Object<'a>>,
}

impl<'a> CompositeObject<'a> {
    fn new() -> CompositeObject<'a> {
        CompositeObject {
            attributes: HashMap::new(),
        }
    }

    fn add_attribute(&mut self, kb: &'a KnowledgeBase, name: &String) -> &'a Object {
        let attribute = Object::new();
        let reference = kb.hold_this(attribute);
        self.attributes.insert(name.clone(), reference);
        return reference;
    }

    fn assign_attribute(&mut self, name: &String, value: &'a Object) {
        self.attributes.insert(name.clone(), value);
    }
}

struct IterableObject<'a>  {
    element: &'a Object<'a>,
}

impl<'a> IterableObject<'a> {
    fn new(element: &'a Object) -> IterableObject<'a> {
        IterableObject {
            element: element
        }
    }
}

struct IndexableObject<'a>  {
    element: &'a Object<'a>,
}

impl<'a> IndexableObject<'a> {
    fn new(element: &'a Object) -> IndexableObject<'a> {
        IndexableObject {
            element: element
        }
    }
}

enum CallableObject<'a> {
    Builtin {semantics: &'a BuiltinFunction},
    Custom {definition: &'a FunctionDefinition},
}

// Giving the compiler something to do
fn main() {
    println!("Hello, world!");

    let x = Object::new();
}
