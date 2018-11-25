use ir::{Constant, Value};

use std::io;

pub struct Printer {}

impl Printer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn print<T>(&self, v: &Value, f: &mut T) -> io::Result<()>
    where
        T: io::Write,
    {
        match v {
            Value::Constant(c) => match c {
                Constant::Number(number) => write!(f, "{}", number),
                Constant::Boolean(tf) => write!(f, "{}", tf),
                Constant::Empty => write!(f, "<empty>"),
            },
            Value::BinOp(op, lhs, rhs) => {
                self.print(lhs, f)?;
                write!(f, " {:?} ", op)?;
                self.print(rhs, f)
            }

            Value::Follow(lhs, rhs) => {
                self.print(lhs, f)?;
                write!(f, ";\n")?;
                self.print(rhs, f)
            }

            Value::Bind(kind, name, rhs) => {
                write!(f, "let {} {} = ", kind, name)?;
                self.print(rhs, f)
            }

            Value::Assign(lhs, rhs) => {
                self.print(lhs, f)?;
                write!(f, " = ")?;
                self.print(rhs, f)
            }

            Value::Variable(name) => write!(f, "{}", name),
            Value::Scope(expr) => {
                write!(f, "{{\n")?;
                self.print(expr, f)?;
                write!(f, "\n}}")
            }
            Value::IfElse(cond, then_expr, else_expr) => {
                self.print(cond, f)?;
                write!(f, " ? ")?;
                self.print(then_expr, f)?;
                write!(f, " : ")?;
                self.print(else_expr, f)
            }
            Value::Apply(func, arg) => {
                self.print(func, f)?;
                write!(f, "(")?;
                self.print(arg, f)?;
                write!(f, ")")
            }
            Value::Function(param, body) => {
                write!(f, "{} -> ", param)?;
                self.print(body, f)
            }
            Value::Typed(ty, candidates, val) => {
                self.print(val, f)?;
                write!(f, " :: {}", ty)?;
                if ! candidates.is_empty() {
                    write!(f, " {:?}", candidates)?;
                }
                Ok(())
            }
        }
    }
}
