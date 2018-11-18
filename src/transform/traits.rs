use ir;

pub trait Transform {
    fn transform(&self, eir: &ir::Value) -> ir::Value;
}
