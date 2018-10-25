use error::LLVMValueNotAvailableError;
use value::Type;

use failure::Error;

use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::types::BasicTypeEnum;

#[derive(Debug)]
enum ValueDataInternal {
    Primitive { internal_value: BasicValueEnum },
    Array { addr: PointerValue, elements: Vec<ValueID> },
    Empty
}

#[derive(Debug)]
pub struct ValueData(TypeID, ValueDataInternal)

impl ValueData {
    pub fn get_type(&self) -> TypeID {
        let ValueData(type_id, ..) = self;
        return type_id;
    }

    pub fn primitive<V>(v: V, t: TypeID) -> Self
        where BasicValueEnum: From<V> {
        ValueData(t, ValueDataInternal::Primitive {
            internal_value: BasicValueEnum::from(v),
        })
    }

    pub fn from_cl<V, T>(v: V, t: T) -> Result<Self, Error>
        where BasicValueEnum: From<V>, BasicTypeEnum: From<T> {
        Ok(ValueData::Primitive {
            internal_value: BasicValueEnum::from(v),
            value_type: Type::from_cl(BasicTypeEnum::from(t))?
        })
    }

    pub fn array(addr: PointerValue, elements: Vec<ValueID>, item_type: Type) -> Self {
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

/// The lightweight and copyable reference to ValueData
#[derive(Clone, Copy, Debug)]
pub struct ValueID(usize);

/// Stores ValueData
#[derive(Debug)]
pub struct ValueStore {
    data: HashMap<ValueID, ValueData>
}

impl ValueStore {
    pub fn new() -> Self {
        ValueStore {
            data: HashMap::new(),
        }
    }

    pub fn new_value(&mut self, data: ValueData) -> ValueID {
        let id = ValueID(self.data.len());
        self.data.insert(id, data);
        id
    }

    pub fn get(&self, id: ValueID) -> Option<&ValueData> {
        self.data.get(id)
    }

    pub fn release(&mut self) {
        self.data.clear()
    }
}

