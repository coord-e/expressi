use super::{Type, Value};
use crate::error::InternalError;
use crate::transform::Transform;

use failure::Error;

use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Node {
    pub value: Value,
    pub type_: Option<Type>,
    pub instantiation_table: HashMap<Type, Node>,
}

impl Deref for Node {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Node {
    pub fn new(value: Value, type_: Type, ty_table: HashMap<Type, Node>) -> Self {
        Node {
            value,
            type_: Some(type_),
            instantiation_table: ty_table,
        }
    }

    pub fn new_untyped(value: Value) -> Self {
        Node {
            value,
            type_: None,
            instantiation_table: HashMap::new(),
        }
    }

    pub fn apply<T>(&self, mut transformer: T) -> Result<Self, Error>
    where
        T: Transform,
    {
        transformer.transform(self)
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn ty_table(&self) -> &HashMap<Type, Node> {
        &self.instantiation_table
    }

    pub fn with_type(self, ty: Type) -> Result<Node, Error> {
        match self.type_() {
            Some(_) => Err(InternalError::AlreadyTyped.into()),
            None => Ok(Node {
                type_: Some(ty),
                ..self
            }),
        }
    }
}
