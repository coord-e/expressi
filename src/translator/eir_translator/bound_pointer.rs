use super::atom::Atom;
use crate::ir::BindingKind;

use inkwell::values;

#[derive(Debug, Clone)]
pub struct BoundPointer {
    kind: BindingKind,
    ptr: Atom<values::PointerValue>,
}

impl BoundPointer {
    pub fn new(kind: BindingKind, ptr: Atom<values::PointerValue>) -> Self {
        Self { kind, ptr }
    }

    pub fn ptr_value(&self) -> &Atom<values::PointerValue> {
        &self.ptr
    }

    pub fn kind(&self) -> BindingKind {
        self.kind
    }
}
