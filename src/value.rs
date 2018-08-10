use cranelift::prelude;
use cranelift::prelude::codegen::ir::dfg::DataFlowGraph;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    Boolean
}

impl Type {
    pub fn from(t: prelude::Type) -> Result<Self, String> {
        Ok(match t {
            prelude::types::I64 => Type::Number,
            prelude::types::B1  => Type::Boolean,
            _ => return Err("There is no representation of this cranelift IR type".to_owned())
        })
    }

    pub fn cl_type(&self) -> Option<prelude::Type> {
        Some(match self {
            Type::Number => prelude::types::I64,
            Type::Boolean => prelude::types::B1,
        })
    }
}

#[derive(Clone)]
pub struct Value<'a> {
    cranelift_value: prelude::Value,
    value_type: Type,
}

impl Value {
    pub fn new(v: prelude::Value, t: prelude::Type) -> Self {
        Value {
            cranelift_value: v,
            value_type: Type::from(t).unwrap()
        }
    }

    pub fn cl_value(&self) -> prelude::Value {
        self.cranelift_value
    }

    pub fn get_type(&self) -> Type {
        self.value_type
    }
}

