use error::TranslationError;

use failure::Error;
use inkwell::types::{BasicTypeEnum, IntType};
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

impl TypeData {
    pub fn from_cl(t: BasicTypeEnum) -> Result<Self, Error> {
        Ok(match t {
            BasicTypeEnum::IntType(int) => match int.get_bit_width() {
                1 => TypeData::Boolean,
                64 => TypeData::Number,
                _ => unimplemented!(),
            },
            _ => {
                return Err(TranslationError::LLVMTypeConversion {
                    from: format!("{:?}", t),
                }.into())
            }
        })
    }

    pub fn cl_type(&self) -> Result<BasicTypeEnum, Error> {
        Ok(match self {
            TypeData::Number => IntType::i64_type(),
            TypeData::Boolean => IntType::bool_type(),
            _ => {
                return Err(TranslationError::InternalTypeConversion {
                    type_description: format!("{:?}", self),
                }.into())
            }
        }.into())
    }

    pub fn size(&self) -> usize {
        match self {
            TypeData::Number => 8,
            TypeData::Boolean => 1,
            TypeData::Array(_, _) => unimplemented!(),
            // TODO: Architecture-independent pointer size
            TypeData::Function(_, _) => 8,
            TypeData::Empty => 0,
            TypeData::Enum(_) => unimplemented!(),
            TypeData::Variable(_) => unimplemented!(),
            TypeData::PolyVariable(_) => unimplemented!(),
        }
    }
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
