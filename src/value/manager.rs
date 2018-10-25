use value::type_::TypeStore;
use value::value::ValueStore;

pub struct ValueManager {
    type_store: TypeStore,
    value_store: ValueStore
}

impl ValueManager {
    pub fn new() -> Self {
        Self {
            type_store: TypeStore::new(),
            value_store: ValueStore::new()
        }
    }

    pub fn new_user_type(&mut self, data: EnumTypeData) -> TypeID {
        self.type_store.new_enum(data)
    }

    pub fn new_value(&mut self, t: TypeID, data: ValueData) -> ValueID {
        self.value_store.new_value(t, data)
    }

}
