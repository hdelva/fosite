#![allow(dead_code)]

use std::collections::HashSet;
use std::collections::HashMap;

/// type aliases

type Type = i16;
type Pointer = i16;

/// Placeholders

trait KnowledgeBase {
    fn link_objects(&mut self, parent: &Pointer, child: &Pointer);

    fn limit_types(&mut self, address: &Pointer, limits: &HashSet<Type>);

    fn function_definition(&self, address: &Pointer) -> &FunctionDefinition;

    fn builtin_function(&self, address: &Pointer) -> &BuiltinFunction;

    fn dereference(&self, address: &Pointer) -> &Object;

    fn allocate(&self) -> Pointer;

    // function belong to a namespace
    fn function(&self, namespace: Namespace, name: &String) -> Option<Callable>;

    // methods belong to a type (~class)
    fn method(&self, class: &Type, name: &String) -> Option<Callable>;

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
struct Object {
    address: Pointer,
    parent: Option<Pointer>,
    //todo, maybe merge iterable and indexable into a collection property
    type_property: Option<TypedObject>,                 // types
    iterable_property: Option<IterableObject>,
    indexable_property: Option<IndexableObject>,
    composite_property: Option<CompositeObject>,    // attributes
}

impl Object {
    fn new(address: Pointer) -> Object {
        Object {
            address: address,
            parent: None,
            type_property: None,
            iterable_property: None,
            indexable_property: None,
            composite_property: None,
        }
    }

    fn set_parent(&mut self, address: Pointer) {
        self.parent = Some(address);
    }

    fn get_types(&self) -> Option<&HashSet<Type>> {
        // an object without any type information can be any type for all we know
        match self.type_property {
            None => return None,
            Some(ref property) => return Some(property.get_types())
        }
    }

    fn limit_types(&mut self, kb: &KnowledgeBase, limits: &HashSet<Type>) {
        // the more information we have about an object, the further we can limit the possible types
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

    fn iterate(&mut self, kb: &KnowledgeBase) -> Pointer {
        // limit the possible types to iterable types
        // return a reference to the object representing its kind of elements
        match self.iterable_property {
            None => {
                self.limit_types(kb, kb.iterable_types());

                let address = kb.allocate();
                let property = IterableObject::new(address);
                self.iterable_property = Some(property);
                return address.clone();
            },
            Some(ref property) => {
                return property.element.clone();
            }
        }
    }

    fn index(&mut self, kb: &KnowledgeBase) -> Pointer {
        // limit the possible types to indexable types
        // return a reference to the object representing its kind of elements
        match self.indexable_property {
            None => {
                self.limit_types(kb, kb.indexable_types());

                let address = kb.allocate();
                let property = IndexableObject::new(address);
                self.indexable_property = Some(property);
                return address.clone()
            },
            Some(ref property) => {
                return property.element.clone();
            }
        }
    }

    fn reference_attribute(&mut self, kb: &mut KnowledgeBase, name: &String) -> Pointer {
        // if the referenced attribute came from a previous assignment
        //   we get no new type information
        // if not we can limit the possible types to types that have this attribute
        match self.composite_property {
            None => {
                self.composite_property = Some(CompositeObject::new());
            },
            Some(ref property) => {
                match property.attributes.get(name){
                    // the referenced property was part of a previous assignment
                    // just return it
                    Some(attribute) => return attribute.clone(),
                    _ => (),
                }
            },
        };

        self.limit_types(kb, kb.types_with_attribute(name));

        return self.composite_property.as_mut().unwrap().add_attribute(kb, name, self.address.clone());
    }

    fn assign_attribute(&mut self, name: &String, value: Pointer) {
        // sets the attribute reference
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

    fn call(&mut self, kb: &mut KnowledgeBase, name: &String) -> Vec<Callable> {
        // ask the knowledge base which methods exists for all the current types
        let mut result = Vec::new();

        match self.parent {
            Some(parent_address) => {
                let mut possible_types = HashSet::new();

                // parent needs to go out of scope
                {
                    let parent = kb.dereference(&parent_address);
                    for t in parent.get_types().unwrap_or(kb.all_types()) {
                        match kb.method(t, name) {
                            None => {},
                            Some(method) => {
                                result.push(method);
                                possible_types.insert(t.clone());
                            },
                        }
                    }
                }

                kb.limit_types(&parent_address, &possible_types);
            },
            None => {
                //todo when we have namespaces
                // match kb.function
            }
        }

        return result;
    }

    // objects with no possible types are invalid
    // code might still function by pure luck, but bad style regardless
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

struct CompositeObject {
    attributes: HashMap<String, Pointer>,
}

impl CompositeObject {
    fn new() -> CompositeObject {
        CompositeObject {
            attributes: HashMap::new(),
        }
    }

    fn add_attribute(&mut self, kb: &mut KnowledgeBase, name: &String, parent: Pointer) -> Pointer {
        let address = kb.allocate();
        kb.link_objects(&parent, &address);
        self.attributes.insert(name.clone(), address);
        return address.clone();
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

struct CallableObject {
    possibilities: HashMap<Type, Callable>,
}

impl CallableObject {
    fn new(possibilities: HashMap<Type, Callable>) -> CallableObject {
        CallableObject {
            possibilities: possibilities,
        }
    }

    fn get_possibilities(&self) -> &HashMap<Type, Callable> {
        return &self.possibilities;
    }

    fn limit_possibilities(&mut self, limits: &HashSet<Type>) {
        let mut new = HashMap::new();

        for (key, value) in self.possibilities.drain() {
            if limits.contains(&key) {
                new.insert(key, value);
            }
        }

        self.possibilities = new;
    }
}

enum Callable {
    Builtin {semantics: Pointer},
    Custom {definition: Pointer},
}

// Giving the compiler something to do
fn main() {
    println!("Hello, world!");

}
