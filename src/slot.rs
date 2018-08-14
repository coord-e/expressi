use cranelift::codegen::ir::entities::StackSlot;

#[derive(Clone, Copy, Debug)]
pub struct Slot {
    ss: StackSlot,
    size: u32
}

impl Slot {
    pub fn new(ss: StackSlot, size: u32) -> Self {
        Slot {ss, size}
    }

    pub fn cl_slot(&self) -> StackSlot {
        self.ss
    }
}
