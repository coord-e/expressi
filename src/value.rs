use error::LLVMValueNotAvailableError;
use type_::Type;

use failure::Error;

use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::types::BasicTypeEnum;

#[derive(Debug)]
pub enum ValueData {
    Primitive { internal_value: BasicValueEnum, value_type: Type },
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

    pub fn primitive<V>(v: V, t: Type) -> Self
        where BasicValueEnum: From<V> {
        ValueData::Primitive {
            internal_value: BasicValueEnum::from(v),
            value_type: t
        }
    }

    pub fn from_cl<V, T>(v: V, t: T) -> Result<Self, Error>
        where BasicValueEnum: From<V>, BasicTypeEnum: From<T> {
        Ok(ValueData::Primitive {
            internal_value: BasicValueEnum::from(v),
            value_type: Type::from_cl(BasicTypeEnum::from(t))?
        })
    }

    pub fn array(addr: PointerValue, elements: Vec<Value>, item_type: Type) -> Self {
        ValueData::Array {
            addr, elements, item_type
        }
    }

    pub fn cl_value(&self) -> Result<BasicValueEnum, Error> {
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
