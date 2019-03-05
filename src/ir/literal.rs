use super::{Identifier, Node, Type};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Literal {
    Number(i64),
    Boolean(bool),
    Function(Identifier, Box<Node>, HashMap<Identifier, Type>),
    Empty,
}
