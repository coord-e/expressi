use super::error::CheckCaptureError;
use super::Transform;
use crate::ir;

use failure::Error;

pub struct CheckCapture;

fn collect_vars(
    eir: &ir::Node,
) -> Result<Box<dyn Iterator<Item = (ir::Identifier, ir::Type)>>, Error> {
    let ty = eir.type_().ok_or(CheckCaptureError::NotTyped)?;
    Ok(match &eir.value {
        ir::Value::Variable(ident) => box vec![(ident.clone(), ty.clone())].into_iter(),
        ir::Value::Literal(c) => match c {
            ir::Literal::Function(ident, box body, captures) => {
                let ident = ident.clone();
                box collect_vars(body)?
                    .chain(captures.clone().into_iter())
                    .filter(move |(e, _)| *e != ident)
            }
            _ => box vec![].into_iter(),
        },
        ir::Value::Let(_, ident, box v, box body) => {
            let ident = ident.clone();
            box collect_vars(v)?
                .chain(collect_vars(body)?)
                .filter(move |(e, _)| *e != ident)
        }
        ir::Value::Assign(box lhs, box rhs) => box collect_vars(lhs)?.chain(collect_vars(rhs)?),
        ir::Value::Follow(box lhs, box rhs) => box collect_vars(lhs)?.chain(collect_vars(rhs)?),
        ir::Value::Apply(box lhs, box rhs) => box collect_vars(lhs)?.chain(collect_vars(rhs)?),
        ir::Value::BinOp(_, box lhs, box rhs) => box collect_vars(lhs)?.chain(collect_vars(rhs)?),
        ir::Value::IfElse(box cond, box then_, box else_) => box collect_vars(cond)?
            .chain(collect_vars(then_)?)
            .chain(collect_vars(else_)?),
    })
}

impl CheckCapture {
    pub fn new() -> Self {
        CheckCapture
    }
}

impl Transform for CheckCapture {
    fn transform_literal(&mut self, lit: &ir::Literal) -> Result<ir::Value, Error> {
        Ok(ir::Value::Literal(match lit {
            ir::Literal::Function(ident, box body, _) => ir::Literal::Function(
                ident.to_string(),
                box self.transform(body)?,
                collect_vars(body)?.filter(|(e, _)| e != ident).collect(),
            ),
            _ => lit.clone(),
        }))
    }
}
