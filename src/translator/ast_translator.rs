use crate::expression::Expression;
use crate::ir::{Constant, Value};

use failure::Error;

use std::collections::HashMap;

pub fn translate_ast(expr: Expression) -> Result<Value, Error> {
    Ok(match expr {
        Expression::Number(number) => Value::Constant(Constant::Number(number)),
        Expression::Boolean(value) => Value::Constant(Constant::Boolean(value)),
        Expression::Empty => Value::Constant(Constant::Empty),
        Expression::Function(ident, body) => {
            let body = translate_ast(*body)?;
            Value::Function(ident, Box::new(body), HashMap::new())
        }
        Expression::Array(_) => unimplemented!(),
        Expression::Type(_) => unimplemented!(),
        Expression::BinOp(op, lhs, rhs) => {
            let lhs = translate_ast(*lhs)?;
            let rhs = translate_ast(*rhs)?;
            Value::BinOp(op, Box::new(lhs), Box::new(rhs))
        }
        Expression::Apply(lhs, rhs) => {
            let rhs_value = translate_ast(*rhs)?;
            let lhs_value = translate_ast(*lhs)?;
            Value::Apply(Box::new(lhs_value), Box::new(rhs_value))
        }
        Expression::Follow(box lhs, box rhs) => match lhs {
            Expression::Bind(kind, name, box bound_value) => {
                let bound_value = translate_ast(bound_value)?;
                let body = translate_ast(rhs)?;
                Value::Let(kind, name, box bound_value, box body)
            }
            _ => {
                let lhs = translate_ast(lhs)?;
                let rhs = translate_ast(rhs)?;
                Value::Follow(box lhs, box rhs)
            }
        },
        Expression::Bind(kind, name, box rhs) => {
            let rhs = translate_ast(rhs)?;
            Value::Let(kind, name.clone(), box rhs, box Value::Variable(name))
        }
        Expression::Assign(lhs, rhs) => {
            let rhs_value = translate_ast(*rhs)?;
            let lhs_value = translate_ast(*lhs)?;
            Value::Assign(Box::new(lhs_value), Box::new(rhs_value))
        }
        Expression::TypeIdentifier(_) => unimplemented!(),
        Expression::Identifier(name) => Value::Variable(name),
        Expression::Cast(_lhs, _rhs) => unimplemented!(),
        Expression::Scope(box expr) => translate_ast(expr)?,
        Expression::IfElse(cond_expr, then_expr, else_expr) => {
            let cond_value = translate_ast(*cond_expr)?;
            let then_value = translate_ast(*then_expr)?;
            let else_value = translate_ast(*else_expr)?;
            Value::IfElse(
                Box::new(cond_value),
                Box::new(then_value),
                Box::new(else_value),
            )
        }
    })
}
