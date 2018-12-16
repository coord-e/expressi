use error::InternalError;
use expression::Operator;
use transform::type_infer::Type;
use transform::Transform;

use failure::Error;

use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(PartialEq, Debug, Clone, Eq, Copy)]
pub enum BindingKind {
    Mutable,
    Immutable,
}

impl fmt::Display for BindingKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BindingKind::Mutable => "mut",
                BindingKind::Immutable => "",
            }
        )
    }
}

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
    Function(Identifier, Box<Value>, HashMap<Identifier, Type>),
    Typed(Type, HashMap<Type, Value>, Box<Value>),
}

impl Value {
    pub fn apply<T>(&self, mut transformer: T) -> Result<Self, Error>
    where
        T: Transform,
    {
        transformer.transform(self)
    }

    pub fn type_(&self) -> Option<&Type> {
        match self {
            Value::Typed(t, ..) => Some(t),
            _ => None,
        }
    }

    pub fn with_type(&self, ty: Type) -> Result<Value, Error> {
        match self {
            Value::Typed(..) => Err(InternalError::AlreadyTyped.into()),
            _ => Ok(Value::Typed(ty, HashMap::new(), box self.clone())),
        }
    }
}
