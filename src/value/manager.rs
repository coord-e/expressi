use error::InternalError;
use value::type_::{EnumTypeData, TypeData, TypeID, TypeStore};
use value::value::{ValueData, ValueID, ValueStore};

use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use failure::Error;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    Number,
    Boolean,
    Empty,
}

pub struct ValueManager {
    type_store: TypeStore,
    value_store: ValueStore,
    primitive_types: HashMap<PrimitiveKind, TypeID>,
    // TODO: Remove `Option` with better initialization
    empty_value: Option<ValueID>,
}

pub type ValueManagerRef = Rc<RefCell<ValueManager>>;

impl ValueManager {
    pub fn new() -> Self {
        let mut manager = Self {
            type_store: TypeStore::new(),
            value_store: ValueStore::new(),
            primitive_types: HashMap::new(),
            empty_value: None,
        };

        let number_t_id = manager.type_store.new_type(TypeData::Number);
        let boolean_t_id = manager.type_store.new_type(TypeData::Boolean);
        let empty_t_id = manager.type_store.new_type(TypeData::Empty);

        manager
            .primitive_types
            .insert(PrimitiveKind::Number, number_t_id);
        manager
            .primitive_types
            .insert(PrimitiveKind::Boolean, boolean_t_id);
        manager
            .primitive_types
            .insert(PrimitiveKind::Empty, empty_t_id);

        manager.empty_value = Some(manager.new_value(empty_t_id, ValueData::Empty));

        manager
    }

    pub fn new_user_type(&mut self, data: EnumTypeData) -> TypeID {
        self.type_store.new_enum(data)
    }

    pub fn new_value(&mut self, t: TypeID, data: ValueData) -> ValueID {
        self.value_store.new_value(t, data)
    }

    pub fn empty_value(&self) -> ValueID {
        self.empty_value.unwrap()
    }

    pub fn type_of(&self, v: ValueID) -> Result<TypeID, Error> {
        self.value_store
            .get(v)
            .map(|data| data.get_type())
            .ok_or(InternalError::InvalidValueID.into())
    }

    pub fn primitive_type(&self, kind: PrimitiveKind) -> TypeID {
        self.primitive_types.get(&kind).unwrap().clone()
    }

    fn primitive_type_llvm(&self, t: BasicTypeEnum) -> TypeID {
        match t {
            BasicTypeEnum::IntType(t) => match t.get_bit_width() {
                1 => self.primitive_type(PrimitiveKind::Boolean),
                64 => self.primitive_type(PrimitiveKind::Number),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    pub fn new_value_from_llvm<V, T>(&mut self, v: V, t: T) -> Result<ValueID, Error>
    where
        BasicValueEnum: From<V>,
        BasicTypeEnum: From<T>,
    {
        let t = self.primitive_type_llvm(BasicTypeEnum::from(t));
        Ok(self.new_value(
            t,
            ValueData::Primitive {
                internal_value: BasicValueEnum::from(v),
            },
        ))
    }

    pub fn llvm_value(&self, v: ValueID) -> Result<BasicValueEnum, Error> {
        self.value_store
            .get(v)
            .ok_or(InternalError::InvalidValueID.into())
            .and_then(|v| v.cl_value())
    }

    pub fn llvm_type(&self, v: TypeID) -> Result<BasicTypeEnum, Error> {
        self.type_store
            .get(v)
            .ok_or(InternalError::InvalidTypeID.into())
            .and_then(|v| v.cl_type())
    }
}
