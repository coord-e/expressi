use error::{
    CraneValueNotAvailableError, CraneliftTypeConversionError, InternalTypeConversionError,
};

use std::fmt;
use std::str::FromStr;

use failure::Error;

use cranelift::prelude;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Type {
    Number,
    Boolean,
    Empty,
}

impl Type {
    pub fn from_cl(t: prelude::Type) -> Result<Self, CraneliftTypeConversionError> {
        Ok(match t {
            prelude::types::I64 => Type::Number,
            prelude::types::B1 => Type::Boolean,
            _ => return Err(CraneliftTypeConversionError { from: t }),
        })
    }

    pub fn cl_type(&self) -> Result<prelude::Type, InternalTypeConversionError> {
        Ok(match self {
            Type::Number => prelude::types::I64,
            Type::Boolean => prelude::types::B1,
            _ => return Err(InternalTypeConversionError { from: *self }),
        })
    }

    pub fn size(&self) -> u32 {
        match self {
            Type::Number => 1,
            Type::Boolean => 1,
            Type::Empty => 0
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rep = match self {
            Type::Number => "Number",
            Type::Boolean => "Boolean",
            Type::Empty => "Empty",
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

#[derive(Clone, Copy)]
pub enum Value {
    Primitive { cranelift_value: prelude::Value, value_type: Type },
    Empty
}

impl Value {
    pub fn get_type(self) -> Type {
        match self {
            Value::Primitive{value_type, ..} => value_type,
            Value::Empty => Type::Empty
        }
    }

    pub fn primitive(v: prelude::Value, t: Type) -> Self {
        Value::Primitive {
            cranelift_value: v,
            value_type: t
        }
    }

    pub fn from_cl(v: prelude::Value, t: prelude::Type) -> Result<Self, Error> {
        Ok(Value::Primitive {
            cranelift_value: v,
            value_type: Type::from_cl(t)?
        })
    }

    pub fn cl_value(self) -> Result<prelude::Value, Error> {
        Ok(match self {
            Value::Primitive {cranelift_value, ..} => cranelift_value,
            _ => return Err(CraneValueNotAvailableError.into())
        })
    }
}
