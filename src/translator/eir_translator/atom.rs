use error::TranslationError;
use transform::type_infer::Type;

use failure::Error;
use inkwell::values::BasicValueEnum;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Atom {
    LLVMValue(BasicValueEnum),
    PolyValue(HashMap<Type, BasicValueEnum>),
}

impl Atom {
    pub fn expect_value(self) -> Result<BasicValueEnum, Error> {
        match self {
            Atom::LLVMValue(v) => Ok(v),
            _ => return Err(TranslationError::ValueExpected.into()),
        }
    }

    pub fn expect_poly_value(self) -> Result<HashMap<Type, BasicValueEnum>, Error> {
        match self {
            Atom::PolyValue(v) => Ok(v),
            _ => return Err(TranslationError::PolyValueExpected.into()),
        }
    }
}

impl From<BasicValueEnum> for Atom {
    fn from(v: BasicValueEnum) -> Self {
        Atom::LLVMValue(v)
    }
}

impl From<HashMap<Type, BasicValueEnum>> for Atom {
    fn from(v: HashMap<Type, BasicValueEnum>) -> Self {
        Atom::PolyValue(v)
    }
}
