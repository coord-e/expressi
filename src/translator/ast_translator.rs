use expression::Expression;
use ir::{Value, Constant};

pub struct ASTTranslator {}

impl ASTTranslator {
    pub fn translate_expr(&mut self, expr: Expression) -> Value {
        match expr {
            Expression::Number(number) => Value::Constant(Constant::Number(number)),
            Expression::Boolean(value) => Value::Constant(Constant::Boolean(value)),
            Expression::Empty => Value::Constant(Constant::Empty),
            Expression::Array(expr) => unimplemented!(),
            Expression::Type(expr) => unimplemented!(),
            Expression::BinOp(op, lhs, rhs) => {
                let lhs = self.translate_expr(*lhs);
                let rhs = self.translate_expr(*rhs);
                Value::BinOp(op, lhs, rhs)
            }
            Expression::Follow(lhs, rhs) => Value::Follow(lhs, rhs),
            Expression::Bind(kind, name, rhs) => Value::Bind(kind, name, rhs),
            Expression::Assign(lhs, rhs) => Value::Assign(lhs, rhs),
            Expression::TypeIdentifier(id) => unimplemented!(),
            Expression::Identifier(name) => Value::Variable(name),
            Expression::Cast(lhs, rhs) => unimplemented!(),
            Expression::Scope(expr) => {
                let content = self.translate_expr(*expr);
                Value::Scope(content)
            }
            Expression::IfElse(cond, then_expr, else_expr) => Value::IfElse(cond, then_expr, else_expr)
        }
    }
}
