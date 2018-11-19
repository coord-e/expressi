use ir::{Constant, Value};
use type_::type_::TypeData;
use type_::{TypeID, TypeStore};
use type_::type_::OperatorKind;

use std::io;

pub struct Printer<'a> {
    type_store: &'a TypeStore,
}

impl<'a> Printer<'a> {
    pub fn new(type_store: &'a TypeStore) -> Self {
        Self { type_store }
    }

    fn print_type<T>(&self, ty: TypeID, f: &mut T) -> io::Result<()>
    where
        T: io::Write,
    {
        match self.type_store.get(ty).unwrap() {
            TypeData::Variable(opt_id) => {
                write!(f, "var(")?;
                match opt_id {
                    Some(id) => self.print_type(*id, f),
                    None => write!(f, "?"),
                }?;
                write!(f, ")")
            }
            TypeData::Operator(kind, types) => {
                match kind {
                    OperatorKind::Function => {
                        self.print_type(types[0], f)?;
                        write!(f, " -> ")?;
                        self.print_type(types[1], f)
                    }
                    _ => {
                        write!(f, "{:?} [", kind)?;
                        for ty in types {
                            self.print_type(*ty, f)?;
                            write!(f, ", ")?;
                        }
                        Ok(())
                    }
                }
            }
            v @ _ => write!(f, "{}", v),
        }
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
            Value::Typed(ty, val) => {
                self.print(val, f)?;
                write!(f, " :: ")?;
                self.print_type(*ty, f)
            }
        }
    }
}
