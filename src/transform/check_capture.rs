use ir;
use transform::Transform;

use failure::Error;

use std::collections::HashSet;

pub struct CheckCapture;

fn collect_vars(eir: &ir::Value) -> Box<dyn Iterator<Item = ir::Identifier>> {
    match eir {
        ir::Value::Variable(ident) => box vec![ident.clone()].into_iter(),
        ir::Value::Constant(_) => box vec![].into_iter(),
        ir::Value::Bind(_, _, box v) => box collect_vars(v),
        ir::Value::Assign(box lhs, box rhs) => box collect_vars(lhs).chain(collect_vars(rhs)),
        ir::Value::Scope(box body) => box collect_vars(body),
        ir::Value::Follow(box lhs, box rhs) => box collect_vars(lhs).chain(collect_vars(rhs)),
        ir::Value::Apply(box lhs, box rhs) => box collect_vars(lhs).chain(collect_vars(rhs)),
        ir::Value::BinOp(_, box lhs, box rhs) => box collect_vars(lhs).chain(collect_vars(rhs)),
        ir::Value::IfElse(box cond, box then_, box else_) => box collect_vars(cond)
            .chain(collect_vars(then_))
            .chain(collect_vars(else_)),
        ir::Value::Function(_, box body, captures) => {
            box collect_vars(body).chain(captures.clone().into_iter())
        }
        ir::Value::Typed(_, _, box value) => box collect_vars(value),
    }
}

impl CheckCapture {
    pub fn new() -> Self {
        CheckCapture
    }
}

impl Transform for CheckCapture {
    fn transform_function(
        &mut self,
        ident: &String,
        body: &ir::Value,
        _: &HashSet<ir::Identifier>,
    ) -> Result<ir::Value, Error> {
        Ok(ir::Value::Function(
            ident.clone(),
            box body.clone(),
            collect_vars(body).collect(),
        ))
    }
}
