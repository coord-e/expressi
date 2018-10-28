use value::type_::{TypeID, TypeStore, TypeData, EnumTypeData};
use value::value::{ValueID, ValueStore, ValueData};
use error::InvalidValueIDError;

use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;

use std::collections::HashMap;

use failure::Error;

pub enum PrimitiveKind {
    Number
}

pub struct ValueManager {
    type_store: TypeStore,
    value_store: ValueStore,
    primitive_types: HashMap<PrimitiveKind, TypeID>
}

impl ValueManager {
    pub fn new() -> Self {
        let mut manager = Self {
            type_store: TypeStore::new(),
            value_store: ValueStore::new(),
            primitive_types: HashMap::new()
        };

        let number_t_id = manager.type_store.new_type(TypeData::Number);
        manager.primitive_types.insert(PrimitiveKind::Number, number_t_id);

        manager
    }

    pub fn new_user_type(&mut self, data: EnumTypeData) -> TypeID {
        self.type_store.new_enum(data)
    }

    pub fn new_value(&mut self, t: TypeID, data: ValueData) -> ValueID {
        self.value_store.new_value(t, data)
    }

    pub fn type_of(&self, v: ValueID) -> Result<TypeID, Error> {
        self.value_store.get(v).map(|data| data.get_type()).ok_or(InvalidValueIDError)
    }

    pub fn primitive_type(&self, kind: PrimitiveKind) -> TypeID {
        self.primitive_types.get(kind).unwrap()
    }

    fn primitive_type_llvm(&self, t: BasicTypeEnum) -> TypeID {
        match t {
            BasicTypeEnum::IntType(_) => self.primitive_types.get(PrimitiveKind::Number).unwrap(),
            _ => unimplemented!()
        }
    }

    pub fn new_value_from_llvm<V, T>(&mut self, v: V, t: T) -> Result<Self, Error>
        where BasicValueEnum: From<V>, BasicTypeEnum: From<T> {
        let t = self.primitive_type_llvm(BasicTypeEnum::from(t))?;
        self.new_value(t, ValueData::Primitive { internal_value: BasicValueEnum::from(v) })
    }
}
