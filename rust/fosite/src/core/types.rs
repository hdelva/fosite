use super::Pointer;

pub enum Type {
    Concrete {types: Vec<Pointer>},
    Abstract {possibilities: Vec<Pointer>},
}

pub fn make_concrete_type(address: Pointer) -> Type {
    return Type::Concrete{types: vec!(address)}
}

/*
pub fn resolve_type<'a>(arg: &'a Type) -> &'a Vec<Pointer> {
    match arg {
        &Type::Concrete {ref types} => {
            return types
        },
        &Type::Abstract {ref types} => {
            match *types.borrow() {
                _AbstractType::Valid {ref possibilities} => {
                    return possibilities
                },
                _AbstractType::Invalid {ref new} => {
                    return resolve_type(new.as_ref())
                }
            }
        }
    }
}

pub fn resolve_type_mut(arg: &mut Type) -> &mut Vec<Pointer> {
    match arg {
        &mut Type::Concrete {ref mut types} => {
            return types
        },
        &mut Type::Abstract {ref mut types} => {
            let mut borrow = types.borrow_mut();
            match *borrow {
                _AbstractType::Valid {ref mut possibilities} => {
                    return possibilities
                },
                _AbstractType::Invalid {new} => {
                    return resolve_type_mut(new.as_mut())
                }
            }
        }
    }
}

pub fn limit_type(arg: &mut Type, possibilities: &HashSet<Pointer>) {
    let mut current = resolve_type_mut(arg);

    current.retain(|t| possibilities.contains(t));
}
*/