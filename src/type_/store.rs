use type_::type_::{EnumTypeData, TypeData};
use type_::PrimitiveKind;

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeID(usize);

pub struct TypeStore {
    data: HashMap<TypeID, TypeData>,
    primitives: HashMap<PrimitiveKind, TypeID>,
}

impl TypeStore {
    pub fn new() -> Self {
        let mut inst = Self {
            data: HashMap::new(),
            primitives: HashMap::new(),
        };

        let number_ty = inst.new_type(TypeData::Number);
        let boolean_ty = inst.new_type(TypeData::Boolean);
        let empty_ty = inst.new_type(TypeData::Empty);

        inst.primitives.insert(PrimitiveKind::Number, number_ty);
        inst.primitives.insert(PrimitiveKind::Boolean, boolean_ty);
        inst.primitives.insert(PrimitiveKind::Empty, empty_ty);

        inst
    }

    pub fn new_function(&mut self, param_type: TypeID, ret_type: TypeID) -> TypeID {
        self.new_type(TypeData::Function(param_type, ret_type))
    }

    pub fn new_type(&mut self, data: TypeData) -> TypeID {
        let id = TypeID(self.data.len());
        self.data.insert(id, data);
        id
    }

    pub fn new_variable(&mut self) -> TypeID {
        self.new_type(TypeData::Variable(None))
    }

    pub fn new_enum(&mut self, data: EnumTypeData) -> TypeID {
        self.new_type(TypeData::Enum(data))
    }

    pub fn get(&self, id: TypeID) -> Option<&TypeData> {
        self.data.get(&id)
    }

    pub fn get_mut(&mut self, id: TypeID) -> Option<&mut TypeData> {
        self.data.get_mut(&id)
    }

    pub fn primitive(&self, kind: PrimitiveKind) -> TypeID {
        self.primitives.get(&kind).unwrap().clone()
    }
}
