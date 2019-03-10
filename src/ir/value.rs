use super::{BindingKind, Identifier, Literal, Node, Type};
use crate::expression::Operator;

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
    pub fn typed_node(self, ty: Type) -> Node {
        Node::new(self, ty, HashMap::new())
    }

    pub fn untyped_node(self) -> Node {
        Node::new_untyped(self)
    }
}
