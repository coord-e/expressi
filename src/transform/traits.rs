use crate::expression::Operator;
use crate::ir;

use failure::Error;

use std::collections::HashMap;

pub trait Transform {
    fn transform(&mut self, eir: &ir::Node) -> Result<ir::Node, Error> {
        let value = match eir.value() {
            ir::Value::Variable(ident) => self.transform_variable(ident)?,
            ir::Value::Literal(c) => self.transform_literal(c)?,
            ir::Value::Let(kind, ident, box value, box body) => {
                let value = self.transform(value)?;
                let body = self.transform(body)?;
                self.transform_let(*kind, ident, &value, &body)?
            }
            ir::Value::Assign(box lhs, box rhs) => {
                let lhs = self.transform(lhs)?;
                let rhs = self.transform(rhs)?;
                self.transform_assign(&lhs, &rhs)?
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
        };

        let instantiation_table = eir
            .ty_table()
            .iter()
            .map(|(t, v)| Ok((t.clone(), self.transform(&v)?)))
            .collect::<Result<HashMap<_, _>, Error>>()?;
        Ok(match eir.type_() {
            Some(ty) => ir::Node::new(value, ty.clone(), instantiation_table),
            None => ir::Node::new_untyped(value), // TODO: Ensure instantiation table is empty
        })
    }

    fn transform_variable(&mut self, ident: &str) -> Result<ir::Value, Error> {
        Ok(ir::Value::Variable(ident.to_string()))
    }

    fn transform_literal(&mut self, c: &ir::Literal) -> Result<ir::Value, Error> {
        Ok(ir::Value::Literal(c.clone()))
    }

    fn transform_let(
        &mut self,
        kind: ir::BindingKind,
        ident: &str,
        v: &ir::Node,
        body: &ir::Node,
    ) -> Result<ir::Value, Error> {
        Ok(ir::Value::Let(
            kind,
            ident.to_string(),
            box v.clone(),
            box body.clone(),
        ))
    }

    fn transform_assign(&mut self, lhs: &ir::Node, rhs: &ir::Node) -> Result<ir::Value, Error> {
        Ok(ir::Value::Assign(box lhs.clone(), box rhs.clone()))
    }

    fn transform_follow(&mut self, lhs: &ir::Node, rhs: &ir::Node) -> Result<ir::Value, Error> {
        Ok(ir::Value::Follow(box lhs.clone(), box rhs.clone()))
    }

    fn transform_apply(&mut self, lhs: &ir::Node, rhs: &ir::Node) -> Result<ir::Value, Error> {
        Ok(ir::Value::Apply(box lhs.clone(), box rhs.clone()))
    }

    fn transform_binop(
        &mut self,
        op: Operator,
        lhs: &ir::Node,
        rhs: &ir::Node,
    ) -> Result<ir::Value, Error> {
        Ok(ir::Value::BinOp(op, box lhs.clone(), box rhs.clone()))
    }

    fn transform_ifelse(
        &mut self,
        cond: &ir::Node,
        then_: &ir::Node,
        else_: &ir::Node,
    ) -> Result<ir::Value, Error> {
        Ok(ir::Value::IfElse(
            box cond.clone(),
            box then_.clone(),
            box else_.clone(),
        ))
    }
}
