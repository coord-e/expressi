pub mod value;
pub mod type_;
pub mod atom;

pub use value::value::{ValueID, ValueStore, ValueData};
pub use value::type_::{TypeStore, TypeID};
pub use value::atom::Atom;
