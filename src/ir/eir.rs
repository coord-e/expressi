use expression::Operator;
use scope::BindingKind;
use transform::Transform;
use value::TypeID;

#[derive(Debug, Clone)]
pub enum Constant {
    Number(i64),
    Boolean(bool),
    Empty,
}

pub type Identifier = String;

#[derive(Debug, Clone)]
pub enum Value {
    Bind(BindingKind, Identifier, Box<Value>),
    Assign(Box<Value>, Box<Value>),
    Scope(Box<Value>),
    Follow(Box<Value>, Box<Value>),
    BinOp(Operator, Box<Value>, Box<Value>),
    IfElse(Box<Value>, Box<Value>, Box<Value>),
    Variable(Identifier),
    Constant(Constant),
    Typed(TypeID, Box<Value>),
}

impl Value {
    pub fn apply<T>(&self, transformer: T) -> Self
    where
        T: Transform,
    {
        transformer.transform(self)
    }
}
