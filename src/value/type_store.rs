#[derive(Clone, Copy, Debug)]
pub struct TypeID(usize);

#[derive(Debug)]
pub type EnumTypeData = Vec<String, Vec<TypeID>>

#[derive(Debug)]
enum TypeData {
    Number,
    Enum(EnumValueData)
}

pub struct TypeStore {
    types: HashMap<TypeID, TypeData>
}

impl TypeStore {
    fn new() -> Self {
        let mut store = Self { types: HashMap::new() };
        store.new_type(TypeData::Number);
        store
    }

    fn new_type(&mut self, data: TypeData) -> TypeID {
        let id = TypeID(self.types.len());
        self.types.insert(id, TypeData);
        id
    }

    fn new_enum(&mut self, data: EnumTypeData) -> TypeID {
        self.new_type(TypeData::Enum(data))
    }
}


