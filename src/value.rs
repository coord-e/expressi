use error::{CraneliftTypeConversionError, InternalTypeConversionError};

use std::fmt;
use std::str::FromStr;

use failure::Error;

use cranelift::prelude;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Type {
    Number,
    Boolean,
    Empty
}

impl Type {
    pub fn from(t: prelude::Type) -> Result<Self, CraneliftTypeConversionError> {
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
            _ => return Err(TypeParseError)
        })
    }
}

#[derive(Clone, Copy)]
pub struct Value {
    pub cranelift_value: Option<prelude::Value>,
    pub value_type: Type,
}

impl Value {
    pub fn new(v: prelude::Value, t: prelude::Type) -> Result<Self, Error> {
        Ok(Value {
            cranelift_value: Some(v),
            value_type: Type::from(t)?,
        })
    }

    pub fn cl_value(&self) -> Option<prelude::Value> {
        self.cranelift_value.map(|v| v.clone())
    }

    pub fn get_type(&self) -> Type {
        self.value_type.clone()
    }
}
