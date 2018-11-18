use transform::Transform;
use transform::error::TypeInferError;
use value::{ValueManager, TypeID};
use value::manager::PrimitiveKind;
use expression::Operator;
use ir;

use failure::Error;

pub struct TypeInfer {
    manager: ValueManager
}

impl TypeInfer {
    pub fn new() -> Self {
        Self {
            manager: ValueManager::new()
        }
    }

    fn bin_op(&self, op: Operator, lhs: &ir::Value, rhs: &ir::Value) -> Result<ir::Value, Error> {
        let new_inst = ir::Value::BinOp(op, box lhs.clone(), box rhs.clone());
        if lhs.type_().is_none() || rhs.type_().is_none() {
            return Ok(new_inst);
        }

        let number_type = self.manager.primitive_type(PrimitiveKind::Number);
        Ok(match op {
            Operator::Index => unimplemented!(),
            op @ _ => {
                self.check_type(lhs.type_().unwrap(), number_type)?;
                self.check_type(rhs.type_().unwrap(), number_type)?;
                ir::Value::Typed(number_type, box new_inst)
            }
        })
    }

    fn check_type(&self, expected: TypeID, t: TypeID) -> Result<TypeID, Error> {
        ensure!(expected == t, TypeInferError::MismatchedTypes { expected, found: t });
        Ok(t)
    }
}

impl Transform for TypeInfer {
    fn transform(&self, eir: &ir::Value) -> Result<ir::Value, Error> {
        match eir {
            v @ ir::Value::Typed(_, _) => Ok(v.clone()),
            ir::Value::BinOp(op, box lhs, box rhs) => self.bin_op(*op, &self.transform(&lhs)?, &self.transform(&rhs)?),
            _ => unimplemented!()
        }
    }
}

