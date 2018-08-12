use error::InternalTypeConversionError;

use std::fmt;

use failure::Error;

use cranelift::prelude;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Type {
    Number,
    Boolean,
}

impl Type {
    pub fn from(t: prelude::Type) -> Result<Self, InternalTypeConversionError> {
        Ok(match t {
            prelude::types::I64 => Type::Number,
            prelude::types::B1 => Type::Boolean,
            _ => return Err(InternalTypeConversionError { from: t }),
        })
    }

    pub fn cl_type(&self) -> Option<prelude::Type> {
        Some(match self {
            Type::Number => prelude::types::I64,
            Type::Boolean => prelude::types::B1,
        })
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rep = match self {
            Type::Number => "Number",
            Type::Boolean => "Boolean",
        };

        write!(f, "{}", rep)
    }
}

#[derive(Clone, Copy)]
pub struct Value {
    pub cranelift_value: prelude::Value,
    pub value_type: Type,
}

impl Value {
    pub fn new(v: prelude::Value, t: prelude::Type) -> Result<Self, Error> {
        Ok(Value {
            cranelift_value: v,
            value_type: Type::from(t)?,
        })
    }

    pub fn cl_value(&self) -> prelude::Value {
        self.cranelift_value.clone()
    }

    pub fn get_type(&self) -> Type {
        self.value_type.clone()
    }
}
