use error::{
    LLVMValueNotAvailableError, LLVMTypeConversionError, InternalTypeConversionError,
};

use std::fmt;
use std::str::FromStr;
use std::ptr::NonNull;

use failure::Error;
use inkwell::values::{AnyValue, PointerValue};
use inkwell::types::{AnyTypeEnum, IntType};

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
    pub fn from_cl(t: AnyTypeEnum) -> Result<Self, LLVMTypeConversionError> {
        Ok(match t {
            AnyTypeEnum::IntType(int) => match int.get_bit_width() {
                1  => Type::Boolean,
                64 => Type::Number,
                _  => unimplemented!()
            },
            _ => return Err(LLVMTypeConversionError { from: format!("{:?}", t) }),
        })
    }

    pub fn cl_type(&self) -> Result<AnyTypeEnum, InternalTypeConversionError> {
        Ok(match self {
            Type::Number => IntType::i64_type(),
            Type::Boolean => IntType::bool_type(),
            _ => return Err(InternalTypeConversionError { from: *self }),
        })
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

#[derive(Debug)]
pub enum ValueData {
    Primitive { internal_value: Box<AnyValue>, value_type: Type },
    Array { addr: PointerValue, elements: Vec<Value>, item_type: Type },
    Empty
}

impl ValueData {
    pub fn get_type(&self) -> Type {
        match *self {
            ValueData::Primitive{value_type, ..} => value_type,
            ValueData::Array{ref elements, item_type, ..} => Type::Array(Box::into_raw_non_null(Box::new(item_type)), elements.len()),
            ValueData::Empty => Type::Empty
        }
    }

    pub fn primitive(v: AnyValue, t: Type) -> Self {
        ValueData::Primitive {
            internal_value: Box::new(v),
            value_type: t
        }
    }

    pub fn from_cl(v: AnyValue, t: AnyTypeEnum) -> Result<Self, Error> {
        Ok(ValueData::Primitive {
            internal_value: Box::new(v),
            value_type: Type::from_cl(t)?
        })
    }

    pub fn array(addr: PointerValue, elements: Vec<Value>, item_type: Type) -> Self {
        ValueData::Array {
            addr, elements, item_type
        }
    }

    pub fn cl_value(&self) -> Result<Box<AnyValue>, Error> {
        Ok(match *self {
            ValueData::Primitive {internal_value, ..} => internal_value,
            _ => return Err(LLVMValueNotAvailableError.into())
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
