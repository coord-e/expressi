use error::TranslationError;

use failure::Error;
use inkwell::values::BasicValueEnum;

#[derive(Debug, Clone)]
pub enum Atom {
    LLVMValue(BasicValueEnum)
}

impl Atom {
    pub fn expect_value(self) -> Result<BasicValueEnum, Error> {
        match self {
            Atom::LLVMValue(v) => Ok(v),
            _ => return Err(TranslationError::ValueExpected.into()),
        }
    }
}

impl From<BasicValueEnum> for Atom {
    fn from(v: BasicValueEnum) -> Self {
        Atom::LLVMValue(v)
    }
}
