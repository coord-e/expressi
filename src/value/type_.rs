use error::{
    LLVMTypeConversionError, InternalTypeConversionError,
};

use inkwell::types::{BasicTypeEnum, IntType};

use std::fmt;
use std::ptr::NonNull;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Type {
    Number,
    Boolean,
    Array(NonNull<Type>, usize),
    Empty,
}

unsafe impl Send for Type {}
unsafe impl Sync for Type {}

impl Type {
    pub fn from_cl(t: BasicTypeEnum) -> Result<Self, LLVMTypeConversionError> {
        Ok(match t {
            BasicTypeEnum::IntType(int) => match int.get_bit_width() {
                1  => Type::Boolean,
                64 => Type::Number,
                _  => unimplemented!()
            },
            _ => return Err(LLVMTypeConversionError { from: format!("{:?}", t) }),
        })
    }

    pub fn cl_type(&self) -> Result<BasicTypeEnum, InternalTypeConversionError> {
        Ok(match self {
            Type::Number => IntType::i64_type(),
            Type::Boolean => IntType::bool_type(),
            _ => return Err(InternalTypeConversionError { from: *self }),
        }.into())
    }

    pub fn size(&self) -> usize {
        match self {
            Type::Number => 8,
            Type::Boolean => 1,
            Type::Array(t, length) => unsafe {*t.as_ptr()}.size() * length,
            Type::Empty => 0
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rep: String = match self {
            Type::Number => "Number".to_string(),
            Type::Boolean => "Boolean".to_string(),
            Type::Array(t, length) => format!("[{}; {}]", unsafe {*t.as_ptr()}, length),
            Type::Empty => "Empty".to_string(),
        };

        write!(f, "{}", rep)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeParseError;

impl FromStr for Type {
    type Err = TypeParseError;

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        Ok(match x {
            "Number" => Type::Number,
            "Boolean" => Type::Boolean,
            "Empty" => Type::Empty,
            _ => return Err(TypeParseError),
        })
    }
}

