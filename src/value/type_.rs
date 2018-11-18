use error::TranslationError;

use failure::Error;
use inkwell::types::{BasicTypeEnum, IntType};

use std::collections::HashMap;
use std::fmt;
use std::ptr::NonNull;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeID(usize);

pub type EnumTypeData = Vec<(String, Vec<TypeID>)>;

#[derive(Debug)]
pub enum TypeData {
    Number,
    Boolean,
    Array(NonNull<TypeData>, usize),
    Empty,
    Enum(EnumTypeData),
    Variable(Option<TypeID>)
}

unsafe impl Send for TypeID {}
unsafe impl Sync for TypeID {}

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
            TypeData::Empty => 0,
            TypeData::Enum(_) => unimplemented!(),
            TypeData::Variable(_) => unimplemented!(),
        }
    }
}

impl fmt::Display for TypeData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rep: String = match self {
            TypeData::Number => "Number".to_string(),
            TypeData::Boolean => "Boolean".to_string(),
            TypeData::Array(_, _) => unimplemented!(),
            TypeData::Empty => "Empty".to_string(),
            TypeData::Enum(data) => format!("{:?}", data),
            TypeData::Variable(instance) => format!("var({:?})", instance),
        };

        write!(f, "{}", rep)
    }
}

pub struct TypeStore {
    data: HashMap<TypeID, TypeData>,
}

impl TypeStore {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn new_type(&mut self, data: TypeData) -> TypeID {
        let id = TypeID(self.data.len());
        self.data.insert(id, data);
        id
    }

    pub fn new_variable(&mut self) -> TypeID {
        self.new_type(TypeData::Variable(None))
    }

    pub fn new_enum(&mut self, data: EnumTypeData) -> TypeID {
        self.new_type(TypeData::Enum(data))
    }

    pub fn get(&self, id: TypeID) -> Option<&TypeData> {
        self.data.get(&id)
    }
}
