use ir;
use transform::error::CheckCaptureError;
use transform::type_infer::Type;
use transform::Transform;

use failure::Error;

use std::collections::HashMap;

pub struct CheckCapture;

fn collect_vars(
    eir: &ir::Value,
) -> Result<Box<dyn Iterator<Item = (ir::Identifier, Type)>>, Error> {
    Ok(match eir {
        ir::Value::Typed(ty, _, box value) => match value {
            ir::Value::Variable(ident) => box vec![(ident.clone(), ty.clone())].into_iter(),
            ir::Value::Constant(_) => box vec![].into_iter(),
            ir::Value::Bind(_, _, box v) => collect_vars(v)?,
            ir::Value::Assign(box lhs, box rhs) => box collect_vars(lhs)?.chain(collect_vars(rhs)?),
            ir::Value::Scope(box body) => collect_vars(body)?,
            ir::Value::Follow(box lhs, box rhs) => box collect_vars(lhs)?.chain(collect_vars(rhs)?),
            ir::Value::Apply(box lhs, box rhs) => box collect_vars(lhs)?.chain(collect_vars(rhs)?),
            ir::Value::BinOp(_, box lhs, box rhs) => {
                box collect_vars(lhs)?.chain(collect_vars(rhs)?)
            }
            ir::Value::IfElse(box cond, box then_, box else_) => box collect_vars(cond)?
                .chain(collect_vars(then_)?)
                .chain(collect_vars(else_)?),
            ir::Value::Function(ident, box body, captures) => {
                let ident = ident.clone();
                box collect_vars(body)?
                    .chain(captures.clone().into_iter())
                    .filter(move |(e, _)| *e != ident)
            }
            ir::Value::Typed(..) => return Err(CheckCaptureError::DoubleTyped.into()),
        },
        _ => return Err(CheckCaptureError::NotTyped.into()),
    })
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
        _: &HashMap<ir::Identifier, Type>,
    ) -> Result<ir::Value, Error> {
        Ok(ir::Value::Function(
            ident.clone(),
            box body.clone(),
            collect_vars(body)?.filter(|(e, _)| e != ident).collect(),
        ))
    }
}