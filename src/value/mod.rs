pub mod atom;
pub mod type_;
pub mod primitive;

pub use value::atom::Atom;
pub use value::type_::{TypeID, TypeStore};
pub use value::primitive::PrimitiveKind;
