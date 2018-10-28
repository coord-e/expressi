use error::LLVMValueNotAvailableError;
use value::TypeID;

use std::collections::HashMap;

use failure::Error;

use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::types::BasicTypeEnum;

#[derive(Debug)]
pub enum ValueData {
    Primitive { internal_value: BasicValueEnum },
    Array { addr: PointerValue, elements: Vec<ValueID> },
    Empty
}

#[derive(Debug)]
pub struct TypedValueData(TypeID, ValueData);

impl TypedValueData {
    pub fn get_type(&self) -> TypeID {
        let TypedValueData(type_id, ..) = self;
        return type_id;
    }

    pub fn cl_value(&self) -> Result<BasicValueEnum, Error> {
        Ok(match *self {
            ValueData::Primitive {internal_value, ..} => internal_value,
            _ => return Err(LLVMValueNotAvailableError.into())
        })
    }
}

/// The lightweight and copyable reference to ValueData
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ValueID(usize);

/// Stores ValueData
#[derive(Debug)]
pub struct ValueStore {
    data: HashMap<ValueID, TypedValueData>
}

impl ValueStore {
    pub fn new() -> Self {
        ValueStore {
            data: HashMap::new(),
        }
    }

    pub fn new_value(&mut self, t: TypeID, data: ValueData) -> ValueID {
        let id = ValueID(self.data.len());
        self.data.insert(id, TypedValueData(t, data));
        id
    }

    pub fn get(&self, id: ValueID) -> Option<&TypedValueData> {
        self.data.get(id)
    }
}

