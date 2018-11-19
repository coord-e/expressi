use ir::BindingKind;

use inkwell::values;

#[derive(Debug, Clone)]
pub struct BoundPointer {
    kind: BindingKind,
    ptr: values::PointerValue,
}

impl BoundPointer {
    pub fn new(kind: BindingKind, ptr: values::PointerValue) -> Self {
        Self { kind, ptr }
    }

    pub fn ptr_value(&self) -> values::PointerValue {
        self.ptr
    }

    pub fn kind(&self) -> BindingKind {
        self.kind
    }
}
