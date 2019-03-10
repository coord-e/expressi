use super::{Constant, Value};

use std::fmt;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Constant(c) => match c {
                Constant::Number(number) => write!(f, "{}", number),
                Constant::Boolean(tf) => write!(f, "{}", tf),
                Constant::Empty => write!(f, "<empty>"),
            },
            Value::BinOp(op, lhs, rhs) => {
                lhs.fmt(f)?;
                write!(f, " {:?} ", op)?;
                rhs.fmt(f)
            }

            Value::Follow(lhs, rhs) => {
                lhs.fmt(f)?;
                writeln!(f, ";")?;
                rhs.fmt(f)
            }

            Value::Bind(kind, name, rhs) => {
                write!(f, "let {} {} = ", kind, name)?;
                rhs.fmt(f)
            }

            Value::Assign(lhs, rhs) => {
                lhs.fmt(f)?;
                write!(f, " = ")?;
                rhs.fmt(f)
            }

            Value::Variable(name) => write!(f, "{}", name),
            Value::Scope(expr) => {
                writeln!(f, "{{")?;
                expr.fmt(f)?;
                write!(f, "\n}}")
            }
            Value::IfElse(cond, then_expr, else_expr) => {
                cond.fmt(f)?;
                write!(f, " ? ")?;
                then_expr.fmt(f)?;
                write!(f, " : ")?;
                else_expr.fmt(f)
            }
            Value::Apply(func, arg) => {
                func.fmt(f)?;
                write!(f, "(")?;
                arg.fmt(f)?;
                write!(f, ")")
            }
            Value::Function(param, body, captures) => {
                write!(f, "{}", param)?;
                if !captures.is_empty() {
                    write!(f, "({:?})", captures)?;
                }
                write!(f, " -> ")?;
                body.fmt(f)
            }
            Value::Typed(ty, candidates, val) => {
                val.fmt(f)?;
                write!(f, " :: {}", ty)?;
                if !candidates.is_empty() {
                    write!(f, "[")?;
                    for t in candidates.keys() {
                        write!(f, "{}, ", t)?;
                    }
                    write!(f, "]")?;
                }
                Ok(())
            }
        }
    }
}
