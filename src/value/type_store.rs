use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeID(usize);

pub type EnumTypeData = Vec<(String, Vec<TypeID>)>;

#[derive(Debug)]
enum TypeData {
    Number,
    Enum(EnumTypeData)
}

pub struct TypeStore {
    types: HashMap<TypeID, TypeData>
}

impl TypeStore {
    pub fn new() -> Self {
        let mut store = Self { types: HashMap::new() };
        store.new_type(TypeData::Number);
        store
    }

    fn new_type(&mut self, data: TypeData) -> TypeID {
        let id = TypeID(self.types.len());
        self.types.insert(id, data);
        id
    }

    pub fn new_enum(&mut self, data: EnumTypeData) -> TypeID {
        self.new_type(TypeData::Enum(data))
    }
}


