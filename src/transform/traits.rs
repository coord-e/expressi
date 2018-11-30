use expression::Operator;
use ir;
use transform::type_infer::Type;

use failure::Error;

use std::collections::{HashMap, HashSet};

pub trait Transform {
    fn transform(&mut self, eir: &ir::Value) -> Result<ir::Value, Error> {
        Ok(match eir {
            ir::Value::Variable(ident) => self.transform_variable(ident)?,
            ir::Value::Constant(c) => self.transform_constant(c)?,
            ir::Value::Bind(kind, ident, box v) => {
                let v = self.transform(v)?;
                self.transform_bind(*kind, ident, &v)?
            }
            ir::Value::Assign(box lhs, box rhs) => {
                let lhs = self.transform(lhs)?;
                let rhs = self.transform(rhs)?;
                self.transform_assign(&lhs, &rhs)?
            }
            ir::Value::Scope(box body) => {
                let body = self.transform(body)?;
                self.transform_scope(&body)?
            }
            ir::Value::Follow(box lhs, box rhs) => {
                let lhs = self.transform(lhs)?;
                let rhs = self.transform(rhs)?;
                self.transform_follow(&lhs, &rhs)?
            }
            ir::Value::Apply(box lhs, box rhs) => {
                let lhs = self.transform(lhs)?;
                let rhs = self.transform(rhs)?;
                self.transform_apply(&lhs, &rhs)?
            }
            ir::Value::BinOp(op, box lhs, box rhs) => {
                let lhs = self.transform(lhs)?;
                let rhs = self.transform(rhs)?;
                self.transform_binop(*op, &lhs, &rhs)?
            }
            ir::Value::IfElse(box cond, box then_, box else_) => {
                let cond = self.transform(cond)?;
                let then_ = self.transform(then_)?;
                let else_ = self.transform(else_)?;
                self.transform_ifelse(&cond, &then_, &else_)?
            }
            ir::Value::Function(ident, box body, captures) => {
                let body = self.transform(body)?;
                self.transform_function(ident, &body, captures)?
            }
            ir::Value::Typed(ty, candidate, box value) => {
                let value = self.transform(value)?;
                self.transform_typed(ty, candidate, &value)?
            }
        })
    }

    fn transform_variable(&mut self, ident: &String) -> Result<ir::Value, Error> {
        Ok(ir::Value::Variable(ident.clone()))
    }

    fn transform_constant(&mut self, c: &ir::Constant) -> Result<ir::Value, Error> {
        Ok(ir::Value::Constant(c.clone()))
    }

    fn transform_bind(
        &mut self,
        kind: ir::BindingKind,
        ident: &String,
        v: &ir::Value,
    ) -> Result<ir::Value, Error> {
        Ok(ir::Value::Bind(kind, ident.clone(), box v.clone()))
    }

    fn transform_assign(&mut self, lhs: &ir::Value, rhs: &ir::Value) -> Result<ir::Value, Error> {
        Ok(ir::Value::Assign(box lhs.clone(), box rhs.clone()))
    }

    fn transform_scope(&mut self, body: &ir::Value) -> Result<ir::Value, Error> {
        Ok(ir::Value::Scope(box body.clone()))
    }

    fn transform_follow(&mut self, lhs: &ir::Value, rhs: &ir::Value) -> Result<ir::Value, Error> {
        Ok(ir::Value::Follow(box lhs.clone(), box rhs.clone()))
    }

    fn transform_apply(&mut self, lhs: &ir::Value, rhs: &ir::Value) -> Result<ir::Value, Error> {
        Ok(ir::Value::Apply(box lhs.clone(), box rhs.clone()))
    }

    fn transform_binop(
        &mut self,
        op: Operator,
        lhs: &ir::Value,
        rhs: &ir::Value,
    ) -> Result<ir::Value, Error> {
        Ok(ir::Value::BinOp(op, box lhs.clone(), box rhs.clone()))
    }

    fn transform_ifelse(
        &mut self,
        cond: &ir::Value,
        then_: &ir::Value,
        else_: &ir::Value,
    ) -> Result<ir::Value, Error> {
        Ok(ir::Value::IfElse(
            box cond.clone(),
            box then_.clone(),
            box else_.clone(),
        ))
    }

    fn transform_function(
        &mut self,
        ident: &String,
        body: &ir::Value,
        captures: &HashSet<ir::Identifier>,
    ) -> Result<ir::Value, Error> {
        Ok(ir::Value::Function(
            ident.clone(),
            box body.clone(),
            captures.clone(),
        ))
    }

    fn transform_typed(
        &mut self,
        type_: &Type,
        candidates: &HashMap<Type, ir::Value>,
        value: &ir::Value,
    ) -> Result<ir::Value, Error> {
        Ok(ir::Value::Typed(
            type_.clone(),
            candidates.clone(),
            box value.clone(),
        ))
    }
}
