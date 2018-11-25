use expression::Expression;
use ir::{Constant, Value};
use transform::type_infer::Type;

use failure::Error;

pub struct ASTTranslator {}

impl ASTTranslator {
    pub fn translate_expr(&mut self, expr: Expression) -> Result<Value, Error> {
        Ok(match expr {
            Expression::Number(number) => Value::Typed(
                Type::Number,
                Vec::new(),
                Box::new(Value::Constant(Constant::Number(number))),
            ),
            Expression::Boolean(value) => Value::Typed(
                Type::Boolean,
                Vec::new(),
                Box::new(Value::Constant(Constant::Boolean(value))),
            ),
            Expression::Empty => {
                Value::Typed(Type::Empty, Vec::new(), Box::new(Value::Constant(Constant::Empty)))
            }
            Expression::Function(ident, body) => {
                let body = self.translate_expr(*body)?;
                Value::Function(ident, Box::new(body))
            }
            Expression::Array(_) => unimplemented!(),
            Expression::Type(_) => unimplemented!(),
            Expression::BinOp(op, lhs, rhs) => {
                let lhs = self.translate_expr(*lhs)?;
                let rhs = self.translate_expr(*rhs)?;
                Value::BinOp(op, Box::new(lhs), Box::new(rhs))
            }
            Expression::Apply(lhs, rhs) => {
                let rhs_value = self.translate_expr(*rhs)?;
                let lhs_value = self.translate_expr(*lhs)?;
                Value::Apply(Box::new(lhs_value), Box::new(rhs_value))
            }
            Expression::Follow(lhs, rhs) => {
                let rhs_value = self.translate_expr(*rhs)?;
                let lhs_value = self.translate_expr(*lhs)?;
                Value::Follow(Box::new(lhs_value), Box::new(rhs_value))
            }
            Expression::Bind(kind, name, rhs) => {
                let rhs_value = self.translate_expr(*rhs)?;
                Value::Bind(kind, name, Box::new(rhs_value))
            }
            Expression::Assign(lhs, rhs) => {
                let rhs_value = self.translate_expr(*rhs)?;
                let lhs_value = self.translate_expr(*lhs)?;
                Value::Assign(Box::new(lhs_value), Box::new(rhs_value))
            }
            Expression::TypeIdentifier(_) => unimplemented!(),
            Expression::Identifier(name) => Value::Variable(name),
            Expression::Cast(_lhs, _rhs) => unimplemented!(),
            Expression::Scope(expr) => {
                let content = self.translate_expr(*expr)?;
                Value::Scope(Box::new(content))
            }
            Expression::IfElse(cond_expr, then_expr, else_expr) => {
                let cond_value = self.translate_expr(*cond_expr)?;
                let then_value = self.translate_expr(*then_expr)?;
                let else_value = self.translate_expr(*else_expr)?;
                Value::IfElse(
                    Box::new(cond_value),
                    Box::new(then_value),
                    Box::new(else_value),
                )
            }
        })
    }
}
