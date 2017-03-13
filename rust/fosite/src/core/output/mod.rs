// warnings
mod identifier_unsafe;
mod attribute_unsafe;
mod out_of_bounds;
mod type_unsafe;
mod while_loop_change;
mod hetero_collection;
mod for_loop_change;

pub use self::identifier_unsafe::*;
pub use self::attribute_unsafe::*;
pub use self::out_of_bounds::*;
pub use self::type_unsafe::*;
pub use self::while_loop_change::*;
pub use self::hetero_collection::*;
pub use self::for_loop_change::*;

pub use super::*;

// errors
mod identifier_invalid;
mod attribute_invalid;
mod binop_invalid;
mod insert_invalid;
mod index_invalid;

pub use self::identifier_invalid::*;
pub use self::attribute_invalid::*;
pub use self::binop_invalid::*;
pub use self::insert_invalid::*;
pub use self::index_invalid::*;

// message identifiers, used in the hashing
pub const IDENTIFIER_UNSAFE: i16 = 1;
pub const ATTRIBUTE_UNSAFE: i16 = 2;
pub const OUT_OF_BOUNDS: i16 = 3;
pub const TYPE_UNSAFE: i16 = 4;
pub const WHILE_LOOP_CHANGE: i16 = 5;
pub const HETERO_COLLECTION: i16 = 6;
pub const FOR_LOOP_CHANGE: i16 = 7;

pub const IDENTIFIER_INVALID: i16 = -1;
pub const ATTRIBUTE_INVALID: i16 = -2;
pub const BINOP_INVALID: i16 = -3;
pub const INSERT_INVALID: i16 = -4;
pub const INDEX_INVALID: i16 = -5;