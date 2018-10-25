use error::{ValueExpectedError, TypeExpectedError};
use value::value_store::ValueID;
use value::type_store::TypeID;

use failure::Error;

#[derive(Debug)]
pub enum Atom {
    Value(ValueID),
    Type(TypeID)
}

impl Atom {
    pub fn expect_value(self) -> Result<ValueID, Error> {
        match self {
            Atom::Value(v) => Ok(v),
            _ => return Err(ValueExpectedError.into())
        }
    }

    pub fn expect_type(self) -> Result<TypeID, Error> {
        match self {
            Atom::Type(v) => Ok(v),
            _ => return Err(TypeExpectedError.into())
        }
    }
}

impl From<ValueID> for Atom {
    fn from(v: ValueID) -> Self {
        Atom::Value(v)
    }
}

impl From<TypeID> for Atom {
    fn from(v: TypeID) -> Self {
        Atom::Type(v)
    }
}
