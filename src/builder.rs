use error::{InvalidCastError, TypeError, ReleasedValueError, InvalidContextBranchError};
use expression::Operator;
use value::{Type, Value, ValueStore, ValueData};
use scope::{Scope, ScopeStack};

use failure::Error;

use inkwell::{basic_block,builder,module,types,values,IntPredicate};

use std::collections::HashMap;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CondCode {
    Equal,
    NotEqual,
    LessThan,
    GreaterThanOrEqual,
    GreaterThan,
    LessThanOrEqual,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub struct Block {
    ebb: basic_block::BasicBlock,
}

impl Block {
    pub fn cl_ebb(&self) -> basic_block::BasicBlock {
        self.ebb
    }
}

pub struct Builder<'a> {
    value_store: ValueStore,
    inst_builder: &'a mut builder::Builder,
    module: Rc<module::Module>,
    scope_stack: ScopeStack,
    block_table: HashMap<Block, Vec<Type>>
}

impl<'a> Builder<'a> {
    pub fn new(inst_builder: &'a mut builder::Builder, module: Rc<module::Module>) -> Self {
        Builder {
            inst_builder,
            module,
            value_store: ValueStore::new(),
            scope_stack: ScopeStack::new(),
            block_table: HashMap::new()
        }
    }

    pub fn to_cl(&self, v: Value) -> Result<values::AnyValue, Error> {
        self.value_store.get(v).ok_or(ReleasedValueError.into()).and_then(|v| v.cl_value())
    }

    pub fn inst_builder<'short>(&'short mut self) -> &'short mut Builder {
        self.inst_builder
    }

    pub fn value_store<'short>(&'short mut self) -> &'short mut ValueStore {
        &mut self.value_store
    }

    pub fn finalize(&mut self) {
        self.inst_builder.finalize()
    }

    pub fn number_constant(&mut self, v: i64) -> Result<Value, Error> {
        let t = types::IntType::i64_type();
        let data = ValueData::from_cl(t.const_int(v, false), t)?;
        Ok(self.value_store.new_value(data))
    }

    pub fn boolean_constant(&mut self, v: bool) -> Result<Value, Error> {
        let t = types::IntType::bool_type();
        let data = ValueData::from_cl(t.const_int(v, false), t)?;
        Ok(self.value_store.new_value(data))
    }

    pub fn apply_op(&mut self, op: Operator, lhs: Value, rhs: Value) -> Result<Value, Error> {
        match op {
            Operator::Add => self.add(lhs, rhs),
            Operator::Sub => self.sub(lhs, rhs),
            Operator::Mul => self.mul(lhs, rhs),
            Operator::Div => self.div(lhs, rhs),
            Operator::BitAnd => self.bit_and(lhs, rhs),
            Operator::BitXor => self.bit_xor(lhs, rhs),
            Operator::BitOr => self.bit_or(lhs, rhs),
            Operator::Lt => self.cmp(CondCode::LessThan, lhs, rhs),
            Operator::Gt => self.cmp(CondCode::GreaterThan, lhs, rhs),
            Operator::Le => self.cmp(CondCode::LessThanOrEqual, lhs, rhs),
            Operator::Ge => self.cmp(CondCode::GreaterThanOrEqual, lhs, rhs),
            Operator::Eq => self.cmp(CondCode::Equal, lhs, rhs),
            Operator::Ne => self.cmp(CondCode::NotEqual, lhs, rhs),
            Operator::Index => self.index(lhs, rhs),
        }
    }

    pub fn add(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let lhs_cl = self.to_cl(lhs)?;
        let rhs_cl = self.to_cl(rhs)?;
        let res = self
            .inst_builder
            .build_int_add(lhs_cl, rhs_cl, "add");
        let data = ValueData::from_cl(res, types::IntType::i64_type())?;
        Ok(self.value_store.new_value(data))
    }

    pub fn sub(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let lhs_cl = self.to_cl(lhs)?;
        let rhs_cl = self.to_cl(rhs)?;
        let res = self
            .inst_builder
            .build_int_sub(lhs_cl, rhs_cl, "sub");
        let data = ValueData::from_cl(res, types::IntType::i64_type())?;
        Ok(self.value_store.new_value(data))
    }

    pub fn mul(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let lhs_cl = self.to_cl(lhs)?;
        let rhs_cl = self.to_cl(rhs)?;
        let res = self
            .inst_builder
            .build_int_mul(lhs_cl, rhs_cl, "mul");
        let data = ValueData::from_cl(res, types::IntType::i64_type())?;
        Ok(self.value_store.new_value(data))
    }

    pub fn div(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let lhs_cl = self.to_cl(lhs)?;
        let rhs_cl = self.to_cl(rhs)?;
        let res = self
            .inst_builder
            .build_int_unsigned_div(lhs_cl, rhs_cl, "div");
        let data = ValueData::from_cl(res, types::IntType::i64_type())?;
        Ok(self.value_store.new_value(data))
    }

    pub fn bit_and(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let lhs_cl = self.to_cl(lhs)?;
        let rhs_cl = self.to_cl(rhs)?;
        let res = self
            .inst_builder
            .build_and(lhs_cl, rhs_cl, "and");
        let data = ValueData::from_cl(res, types::IntType::i64_type())?;
        Ok(self.value_store.new_value(data))
    }

    pub fn bit_or(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let lhs_cl = self.to_cl(lhs)?;
        let rhs_cl = self.to_cl(rhs)?;
        let res = self
            .inst_builder
            .build_or(lhs_cl, rhs_cl, "or");
        let data = ValueData::from_cl(res, types::IntType::i64_type())?;
        Ok(self.value_store.new_value(data))
    }

    pub fn bit_xor(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let lhs_cl = self.to_cl(lhs)?;
        let rhs_cl = self.to_cl(rhs)?;
        let res = self
            .inst_builder
            .build_xor(lhs_cl, rhs_cl, "xor");
        let data = ValueData::from_cl(res, types::IntType::i64_type())?;
        Ok(self.value_store.new_value(data))
    }

    pub fn cmp(&mut self, cmp_type: CondCode, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let cc = match cmp_type {
            CondCode::Equal              => IntPredicate::EQ,
            CondCode::NotEqual           => IntPredicate::NE,
            CondCode::LessThan           => IntPredicate::SLT,
            CondCode::GreaterThanOrEqual => IntPredicate::SGE,
            CondCode::GreaterThan        => IntPredicate::SGT,
            CondCode::LessThanOrEqual    => IntPredicate::SLE,
        };

        let lhs_cl = self.to_cl(lhs)?;
        let rhs_cl = self.to_cl(rhs)?;
        let res = self
            .inst_builder
            .build_int_compare(cc, lhs_cl, rhs_cl);
        let data = ValueData::from_cl(res, types::bool_type())?;
        Ok(self.value_store.new_value(data))
    }

    pub fn index(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        match lhs.get_type() {
            Type::Array(..) => {},
            _ => return Err(TypeError.into())
        }
        if rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }

        let byte = self.number_constant(8)?;
        let offset = self.mul(rhs, byte)?;
        let offset_cl = self.to_cl(offset)?;
        let data = {
            let lhs_data = self.value_store.get(lhs).ok_or(ReleasedValueError)?;
            if let ValueData::Array { elements, addr, item_type, ..} = lhs_data {
                    let pointed_addr = self.inst_builder.build_int_add(*addr, offset_cl, "idx_offset");
                    let loaded = self.inst_builder.build_load(pointed_addr, "idx_load");
                    ValueData::primitive(loaded, *item_type)
            } else {
                return Err(TypeError.into());
            }
        };
        Ok(self.value_store.new_value(data))
    }

    pub fn declare_var(&mut self, name: &str, t: Type, unique: bool) -> Result<String, Error> {
        let real_name = if unique { self.scope_stack.unique_name(name) } else { name.to_string() };
        let variable = self.inst_builder.build_alloc(t.cl_type()?, name);
        let empty = self.value_store.new_value(ValueData::Empty);
        self.scope_stack.add(real_name, empty, variable); // TODO: TypeValue
        Ok(real_name)
    }

    pub fn set_var(&mut self, name: &str, val: Value) -> Result<Value, Error> {
        let variable = self.scope_stack.get_var(name).unwrap_or({
            let variable = self.inst_builder.build_alloc(val.get_type().cl_type()?, name);
            self.scope_stack.add(name, val, variable);
            variable
        });
        if let Ok(val) = self.to_cl(val) {
            self.inst_builder.build_store(variable, val);
        }
        self.scope_stack.set(name, val);
        Ok(val)
    }

    pub fn get_var(&mut self, name: &str) -> Option<Value> {
        self.scope_stack.get_var(name).map(|var| {
            let value = self.scope_stack.get(name).unwrap();
            let data = ValueData::primitive(self.inst_builder.build_load(var, "load_var"), value.get_type());
            self.value_store.new_value(data)
        })
    }

    pub fn cast_to(&mut self, v: Value, t: Type) -> Result<Value, Error> {
        if v.get_type() == t {
            return Err(InvalidCastError {
                from: v.get_type(),
                to: t,
            }.into());
        }
        Ok(match (v.get_type(), t) {
            (Type::Number, Type::Boolean) => {
                let zero = self.number_constant(0)?;
                self.cmp(CondCode::NotEqual, v, zero)?
            }
            (Type::Boolean, Type::Number) => {
                let cl = self.to_cl(v)?;
                let data = ValueData::primitive(self.inst_builder.build_int_cast(cl, t.cl_type()?), t);
                self.value_store.new_value(data)
            },
            _ => {
                return Err(InvalidCastError {
                    from: v.get_type(),
                    to: t,
                }.into())
            }
        })
    }

    pub fn enter_scope(&mut self, sc: Scope) {
        self.scope_stack.push(sc);
    }

    pub fn exit_scope(&mut self) -> Result<Scope, Error> {
        self.scope_stack.pop()
    }

    pub fn array_alloc(&mut self, t: Type, size: u32) -> Result<values::PointerValue, Error> {
        Ok(self.inst_builder.build_array_alloca(t.to_cl()?, self.number_constant(size)?, "array_alloc"))
    }

    pub fn store(&mut self, v: Value, addr: values::PointerValue, offset: i32) -> Result<(), Error> {
        let ptr = self.inst_builder.build_in_bounds_gep(addr, &[types::IntType::i32_type().const_int(0), types::IntType::i32_type().const_int(offset)], "store");
        let cl = self.to_cl(v)?;
        self.inst_builder.build_store(ptr, cl);
        Ok(())
    }

    pub fn load(&mut self, t: Type, addr: values::PointerValue, offset: i32) -> Result<Value, Error> {
        let ptr = self.inst_builder.build_in_bounds_gep(addr, &[types::IntType::i32_type().const_int(0), types::IntType::i32_type().const_int(offset)], "store");
        let data = ValueData::from_cl(self.inst_builder.build_load(ptr), t.cl_type()?)?;
        Ok(self.value_store.new_value(data))
    }

    pub fn create_block(&mut self) -> Result<Block, Error> {
        let parent = self.inst_builder.get_insert_block().and_then(|b| ins_block.get_parent()).ok_or(InvalidContextBranchError.into())?;
        let block = self.module.get_context().append_basic_block(parent, "");
        Block { ebb: block }
    }

    pub fn brz(&mut self, condition: Value, then_block: Block, else_block: Block) -> Result<(), Error> {
        if condition.get_type() != Type::Boolean {
            return Err(TypeError.into());
        }
        let cl = self.to_cl(condition)?;
        self.inst_builder
            .build_conditional_branch(cl, then_block.cl_ebb(), else_block.cl_ebb());
        Ok(())
    }

    pub fn jump(&mut self, block: Block) {
        self.inst_builder.build_unconditional_branch(block.cl_ebb());
    }

    pub fn switch_to_block(&mut self, block: Block) {
        self.inst_builder.position_at_end(block.cl_ebb());
    }
}
