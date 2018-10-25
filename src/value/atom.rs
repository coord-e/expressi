use error::{ValueExpectedError, TypeExpectedError};
use value::Value;
use value::type_store::TypeID;

use failure::Error;

#[derive(Debug)]
pub enum Atom {
    Value(Value),
    Type(TypeID)
}

impl Atom {
    fn expect_value(self) -> Result<Value, Error> {
        match self {
            Atom::Value(v) => Ok(v),
            _ => return ValueExpectedError
        }
    }

    fn expect_type(self) -> Result<TypeID, Error> {
        match self {
            Atom::Type(v) => Ok(v),
            _ => return TypeExpectedError
        }
    }
}

impl From<Value> for Atom {
    fn from(v: Value) -> Self {
        Atom::Value(v)
    }
}

impl From<TypeID> for Atom {
    fn from(v: TypeID) -> Self {
        Atom::Type(v)
    }
}
