use type_::TypeID;

use std::fmt;
use std::ptr::NonNull;

pub type EnumTypeData = Vec<(String, Vec<TypeID>)>;

#[derive(Debug, Clone)]
pub enum TypeData {
    Number,
    Boolean,
    Array(NonNull<TypeData>, usize),
    Function(TypeID, TypeID),
    Empty,
    Enum(EnumTypeData),
    Variable(Option<TypeID>),
    PolyVariable(Vec<TypeID>),
}

impl fmt::Display for TypeData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rep: String = match self {
            TypeData::Number => "Number".to_string(),
            TypeData::Boolean => "Boolean".to_string(),
            TypeData::Array(_, _) => unimplemented!(),
            TypeData::Function(param, ret) => format!("{:?} -> {:?}", param, ret),
            TypeData::Empty => "Empty".to_string(),
            TypeData::Enum(data) => format!("{:?}", data),
            TypeData::Variable(instance) => format!("var({:?})", instance),
            TypeData::PolyVariable(types) => format!("pvar({:?})", types),
        };

        write!(f, "{}", rep)
    }
}
