pub mod atom;
pub mod primitive;
pub mod type_;

pub use type_::atom::Atom;
pub use type_::primitive::PrimitiveKind;
pub use type_::type_::{TypeID, TypeStore};
