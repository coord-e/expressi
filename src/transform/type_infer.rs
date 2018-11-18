use transform::Transform;
use value::ValueManager;
use ir;

pub struct TypeInfer {
    manager: ValueManager
}

impl TypeInfer {
    pub fn new() -> Self {
        Self {
            manager: ValueManager::new()
        }
    }
}

impl Transform for TypeInfer {
    fn transform(&self, eir: &ir::Value) -> ir::Value {
        eir.clone()
    }
}

