pub mod value;
pub mod type_;
pub mod type_store;
pub mod atom;

pub use value::type_::Type;
pub use value::value::{Value, ValueStore, ValueData};
pub use value::type_store::{TypeStore, TypeID};
pub use value::atom::Atom;
