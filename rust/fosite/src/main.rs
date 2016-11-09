#![allow(dead_code)]
#![feature(box_syntax)]

use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

/// type aliases

type Type = i16;

//todo change to enum; Data -> &Object / Code -> Callable
type Pointer = i16;

/// Placeholders

trait KnowledgeBase {
    fn link_objects(&mut self, parent: &Pointer, child: &Pointer); //

    fn limit_types(&mut self, address: &Pointer, limits: &HashSet<Type>); //

    fn function_definition(&self, address: &Pointer) -> &FunctionDefinition; //

    fn builtin_function(&self, address: &Pointer) -> &BuiltinFunction; //

    fn dereference(&self, address: &Pointer) -> &Object; //

    fn allocate(&mut self) -> Pointer; //

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

struct Scope {
    // depending on the execution path, a variable name can refer to multiple things
    identifiers: HashMap<String, Vec<HashSet<Pointer>>>,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            identifiers: HashMap::new(),
        }
    }

    fn assign_to_identifier(&mut self, name: &String, possibilities: HashSet<Pointer>) {
        match self.identifiers.entry(name.clone()) {
            Vacant(entry) => {
                let list = entry.insert(Vec::new());
                list.push(possibilities);
            },
            Occupied(mut entry) => {
                let list = entry.get_mut();
                list.push(possibilities);
            }
        };
    }

    fn resolve_identifier(&self, name: &String) -> Option<&HashSet<Pointer>> {
        match self.identifiers.get(name) {
            Some(list) => return list.last(),
            None => None,
        }
    }

    // takes ownership of self, dies in the process
    fn merge_into(self, target: &mut Scope) {
        for (name, mut list) in self.identifiers {
            match target.identifiers.entry(name) {
                Vacant(entry) => {
                    let own_list = entry.insert(Vec::new());
                    own_list.append(&mut list);
                },
                Occupied(mut entry) => {
                    let result = Vec::new();

                    // let own_list go out of scope before inserting
                    {
                        // take ownership of the current content, effectively removing it
                        let own_list = entry.get();

                        // improved zip, more efficient as well
                        let longest_list;
                        let shortest_list;

                        if list.len() > own_list.len() {
                            longest_list = &list;
                            shortest_list = own_list;
                        } else {
                            longest_list = own_list;
                            shortest_list = &list;
                        }

                        let ref cut = longest_list[..shortest_list.len()];

                        let mut result = Vec::new();

                        // merge whatever they have in common
                        for i in 0..cut.len() {
                            let ref x = longest_list[i];
                            let ref y = shortest_list[i];

                            let merged: HashSet<Pointer> = x.union(&y).cloned().collect();
                            result.push(merged);
                        }

                        // add the rest
                        for i in cut.len()..longest_list.len() {
                            let x: HashSet<Pointer> = longest_list[i].clone();
                            result.push(x);
                        }
                    }

                    // put the result back in the hashmap
                    entry.insert(result);
                }
            }
        }
    }
}

struct Pls<'a> {
    count: i16,

    parent: Option<&'a Pls<'a>>,

    objects: HashMap<Pointer, Object>,
    function_definitions: HashMap<Pointer, FunctionDefinition>,
    builtin_functions: HashMap<Pointer, BuiltinFunction>,

    private_scope: Scope,
    public_scope: Scope,

    types: HashSet<Type>,
    iterable_types: HashSet<Type>,
    indexable_types: HashSet<Type>,
    types_with_attribute: HashMap<String, Type>,
    methods: HashMap<Type, HashMap<String, Callable>>,
}

impl<'a> Pls<'a> {
    /// add new things to the knowledge base
    fn new_object(&mut self) -> Pointer {
        let index = self.count;
        self.count += 1;

        let object = Object::new(index);
        self.objects.insert(index, object);

        return index;
    }

    fn define_custom_function(&mut self, defintion: FunctionDefinition) {
        let index = self.count;
        self.count += 1;

        self.function_definitions.insert(index, defintion);
    }

    fn define_builtin_function(&mut self, definition: BuiltinFunction) {
        let index = self.count;
        self.count += 1;

        self.builtin_functions.insert(index, definition);
    }

    /// change things in the knowledge base
    //todo remove these
    fn link_objects(&mut self, parent_address: &Pointer, child_address: &Pointer) {
        match self.objects.get_mut(child_address) {
            Some(child) => {
                child.set_parent(parent_address.clone());
            },
            None => panic!("Invalid Pointer Value")
        }
    }

    fn limit_types(&mut self, address: &Pointer, limits: &HashSet<Type>) {
        match self.objects.get_mut(address) {
            Some(object) => {
                object.limit_types(limits);
            },
            None => panic!("Invalid Pointer Value")
        }
    }

    /// get things from the knowledge base
    fn dereference_pointer(&self, address: &Pointer) -> &Object {
        match self.objects.get(address) {
            Some(def) => {
                return def;
            },
            None => panic!("Invalid Pointer Value")
        }
    }

    fn get_function_definition(&self, address: &Pointer) -> &FunctionDefinition {
        match self.function_definitions.get(address) {
            Some(def) => {
                return def;
            },
            None => panic!("Invalid Pointer Value")
        }
    }

    fn get_builtin_function(&self, address: &Pointer) -> &BuiltinFunction {
        match self.builtin_functions.get(address) {
            Some(def) => {
                return def;
            },
            None => panic!("Invalid Pointer Value")
        }
    }

}

//todo implement for each builtin function
struct BuiltinFunction {

}

impl BuiltinFunction {
    //fn call(&self, kb: &mut KnowledgeBase, args: [&Object]);
}

struct FunctionDefinition {

}

struct Namespace {

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

    fn limit_types(&mut self, limits: &HashSet<Type>) {
        // the more information we have about an object, the further we can limit the possible types
        match self.type_property {
            None => {
                let property = TypedObject::new(limits.clone());
                self.type_property = Some(property);
            },
            Some(ref mut property) => {
                property.limit(limits);
            }
        }
    }

    fn iterate(&mut self, kb: &mut KnowledgeBase) -> Pointer {
        // limit the possible types to iterable types
        // return a reference to the object representing its kind of elements
        match self.iterable_property {
            None => {
                self.limit_types(kb.iterable_types());

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

    fn index(&mut self, kb: &mut KnowledgeBase) -> Pointer {
        // limit the possible types to indexable types
        // return a reference to the object representing its kind of elements
        match self.indexable_property {
            None => {
                self.limit_types(kb.indexable_types());

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

        self.limit_types(kb.types_with_attribute(name));

        //todo remove need for self.address, add reference to parent in every property?
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
    //todo split in private/public/(protected)
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
