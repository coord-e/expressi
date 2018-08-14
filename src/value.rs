use error::{
    CraneValueNotAvailableError, CraneliftTypeConversionError, InternalTypeConversionError,
};

use std::fmt;
use std::str::FromStr;
use std::ptr::NonNull;

use failure::Error;

use cranelift::prelude;

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

    pub fn size(&self) -> usize {
        match self {
            Type::Number => 1,
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

#[derive(Debug)]
pub enum ValueData {
    Primitive { cranelift_value: prelude::Value, value_type: Type },
    Array { elements: Vec<Value>, item_type: Type },
    Empty
}

impl ValueData {
    pub fn get_type(&self) -> Type {
        match *self {
            ValueData::Primitive{value_type, ..} => value_type,
            ValueData::Array{ref elements, mut item_type, ..} => Type::Array(NonNull::new(&mut item_type).unwrap(), elements.len()),
            ValueData::Empty => Type::Empty
        }
    }

    pub fn primitive(v: prelude::Value, t: Type) -> Self {
        ValueData::Primitive {
            cranelift_value: v,
            value_type: t
        }
    }

    pub fn from_cl(v: prelude::Value, t: prelude::Type) -> Result<Self, Error> {
        Ok(ValueData::Primitive {
            cranelift_value: v,
            value_type: Type::from_cl(t)?
        })
    }

    pub fn array(elements: Vec<Value>, item_type: Type) -> Self {
        ValueData::Array {
            elements, item_type
        }
    }

    pub fn cl_value(&self) -> Result<prelude::Value, Error> {
        Ok(match *self {
            ValueData::Primitive {cranelift_value, ..} => cranelift_value,
            _ => return Err(CraneValueNotAvailableError.into())
        })
    }
}

/// Stores ValueData
#[derive(Debug)]
pub struct ValueStore {
    data: Vec<ValueData>
}

impl ValueStore {
    pub fn new() -> Self {
        ValueStore {
            data: Vec::new(),
        }
    }

    pub fn new_value(&mut self, data: ValueData) -> Value {
        let t = data.get_type();
        self.data.push(data);
        Value::from_idx(self.data.len() - 1, t)
    }

    pub fn get(&self, rf: Value) -> Option<&ValueData> {
        let Value(idx, ..) = rf;
        if self.data.len() <= idx { None } else { Some(&self.data[idx]) }
    }

    pub fn release(&mut self) {
        self.data.clear()
    }
}


/// The lightweight and copyable reference to ValueData
#[derive(Clone, Copy, Debug)]
pub struct Value(usize, Type);

impl Value {
    fn from_idx(idx: usize, t: Type) -> Self {
        Value(idx, t)
    }

    pub fn get_type(&self) -> Type {
        self.1
    }
}
