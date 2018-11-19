use expression::Operator;
use builder::BindingKind;
use transform::Transform;
use value::TypeID;

use failure::Error;

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
    Apply(Box<Value>, Box<Value>),
    BinOp(Operator, Box<Value>, Box<Value>),
    IfElse(Box<Value>, Box<Value>, Box<Value>),
    Variable(Identifier),
    Constant(Constant),
    Function(Identifier, Box<Value>),
    Typed(TypeID, Box<Value>),
}

impl Value {
    pub fn apply<T>(&self, mut transformer: T) -> Result<Self, Error>
    where
        T: Transform,
    {
        transformer.transform(self)
    }

    pub fn type_(&self) -> Option<TypeID> {
        match self {
            Value::Typed(t, _) => Some(t.clone()),
            _ => None,
        }
    }
}
