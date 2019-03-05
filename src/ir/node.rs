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
    pub instantiation_table: HashMap<Type, Value>,
}

impl Deref for Node {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        &self.value
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
