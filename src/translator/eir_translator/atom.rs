use error::TranslationError;
use type_::TypeID;

use failure::Error;
use inkwell::values::BasicValueEnum;

#[derive(Debug, Clone)]
pub enum Atom {
    LLVMValue(BasicValueEnum),
    Type(TypeID),
}

impl Atom {
    pub fn expect_value(self) -> Result<BasicValueEnum, Error> {
        match self {
            Atom::LLVMValue(v) => Ok(v),
            _ => return Err(TranslationError::ValueExpected.into()),
        }
    }

    pub fn expect_type(self) -> Result<TypeID, Error> {
        match self {
            Atom::Type(v) => Ok(v),
            _ => return Err(TranslationError::TypeExpected.into()),
        }
    }
}

impl From<BasicValueEnum> for Atom {
    fn from(v: BasicValueEnum) -> Self {
        Atom::LLVMValue(v)
    }
}

impl From<TypeID> for Atom {
    fn from(v: TypeID) -> Self {
        Atom::Type(v)
    }
}
