use super::{Literal, Node, Value};

use std::fmt;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Literal(c) => match c {
                Literal::Number(number) => write!(f, "{}", number),
                Literal::Boolean(tf) => write!(f, "{}", tf),
                Literal::Function(param, body, captures) => {
                    write!(f, "{}", param)?;
                    if !captures.is_empty() {
                        write!(f, "({:?})", captures)?;
                    }
                    write!(f, " -> ")?;
                    body.fmt(f)
                }
                Literal::Empty => write!(f, "<empty>"),
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

            Value::Let(kind, name, value, body) => {
                write!(f, "let {} {} = ", kind, name)?;
                value.fmt(f)?;
                write!(f, " in {}", body)
            }

            Value::Assign(lhs, rhs) => {
                lhs.fmt(f)?;
                write!(f, " = ")?;
                rhs.fmt(f)
            }

            Value::Variable(name) => write!(f, "{}", name),
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
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value().fmt(f)?;
        if let Some(ty) = self.type_() {
            write!(f, " :: {}", ty)?;
            if !self.ty_table().is_empty() {
                write!(f, "[")?;
                for t in self.ty_table().keys() {
                    write!(f, "{}, ", t)?;
                }
                write!(f, "]")?;
            }
        }
        Ok(())
    }
}
