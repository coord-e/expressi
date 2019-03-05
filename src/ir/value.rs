use super::{BindingKind, Identifier, Literal, Node, Type};
use crate::expression::Operator;

use failure::Error;

use std::collections::HashMap;

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
