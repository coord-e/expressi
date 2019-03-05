use super::Type;

use crate::error::InternalError;
use crate::expression::Operator;
use crate::transform::Transform;

use failure::Error;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;

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
pub enum Literal {
    Number(i64),
    Boolean(bool),
    Function(Identifier, Box<Node>, HashMap<Identifier, Type>),
    Empty,
}

pub type Identifier = String;

#[derive(Debug, Clone)]
pub enum Value {
    Let(BindingKind, Identifier, Box<Node>, Box<Node>),
    Follow(Box<Node>, Box<Node>),
    Assign(Box<Node>, Box<Node>),
    Apply(Box<Node>, Box<Node>),
    BinOp(Operator, Box<Node>, Box<Node>),
    IfElse(Box<Node>, Box<Node>, Box<Node>),
    Variable(Identifier),
    Literal(Literal),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub value: Value,
    pub type_: Option<Type>,
    pub instantiation_table: HashMap<Type, Value>,
}

impl Deref for Node {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Value {
    pub fn with_type(self, ty: Type) -> Result<Node, Error> {
        Ok(Node {
            value: self,
            type_: Some(ty),
            instantiation_table: HashMap::new(),
        })
    }

    pub fn untyped_node(self) -> Node {
        Node {
            value: self,
            type_: None,
            instantiation_table: HashMap::new(),
        }
    }
}

impl Node {
    pub fn apply<T>(&self, mut transformer: T) -> Result<Self, Error>
    where
        T: Transform,
    {
        transformer.transform(self)
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn with_type(&self, ty: Type) -> Result<Node, Error> {
        match self.type_ {
            Some(_) => Err(InternalError::AlreadyTyped.into()),
            None => Ok(Node {
                type_: Some(ty),
                ..self.clone()
            }),
        }
    }
}
