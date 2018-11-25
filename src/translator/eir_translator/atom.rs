use error::TranslationError;
use transform::type_infer::Type;

use failure::Error;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Atom<T> {
    LLVMValue(T),
    PolyValue(HashMap<Type, T>),
}

impl<T> Atom<T> {
    pub fn expect_value(self) -> Result<T, Error> {
        match self {
            Atom::LLVMValue(v) => Ok(v),
            _ => return Err(TranslationError::ValueExpected.into()),
        }
    }

    pub fn expect_poly_value(self) -> Result<HashMap<Type, T>, Error> {
        match self {
            Atom::PolyValue(v) => Ok(v),
            _ => return Err(TranslationError::PolyValueExpected.into()),
        }
    }
}

impl<T> From<T> for Atom<T> {
    fn from(v: T) -> Self {
        Atom::LLVMValue(v)
    }
}

impl<T> From<HashMap<Type, T>> for Atom<T> {
    fn from(v: HashMap<Type, T>) -> Self {
        Atom::PolyValue(v)
    }
}
