use error::{InvalidCastError, TypeError, ReleasedValueError};
use expression::Operator;
use value::{Type, Value, ValueStore, ValueData};
use scope::{Scope, ScopeStack};
use slot::Slot;

use failure::Error;

use cranelift::codegen::ir::{condcodes, entities, types, InstBuilder, stackslot};
use cranelift::codegen::ir::immediates::Offset32;
use cranelift::prelude::{EntityRef, FunctionBuilder, Variable};

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
    ebb: entities::Ebb,
}

impl Block {
    pub fn cl_ebb(&self) -> entities::Ebb {
        self.ebb
    }
}

pub struct Builder<'a> {
    pub inst_builder: &'a mut FunctionBuilder<'a, Variable>,
    pub scope_stack: ScopeStack,
    pub value_store: ValueStore,
    pub block_table: HashMap<Block, Vec<Type>>
}

impl<'a> Builder<'a> {
    pub fn to_cl(&self, v: Value) -> Result<entities::Value, Error> {
        self.value_store.get(v).ok_or(ReleasedValueError.into()).and_then(|v| v.cl_value())
    }

    pub fn inst_builder<'short>(&'short mut self) -> &'short mut FunctionBuilder<'a, Variable> {
        self.inst_builder
    }

    pub fn finalize(&mut self) {
        self.inst_builder.finalize()
    }

    pub fn number_constant(&mut self, v: i64) -> Result<Value, Error> {
        let t = types::I64;
        let data = ValueData::from_cl(self.inst_builder.ins().iconst(t, v), t)?;
        Ok(self.value_store.new_value(data))
    }

    pub fn boolean_constant(&mut self, v: bool) -> Result<Value, Error> {
        let t = types::B1;
        let data = ValueData::from_cl(self.inst_builder.ins().bconst(t, v), t)?;
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
            .ins()
            .iadd(lhs_cl, rhs_cl);
        let data = ValueData::from_cl(res, types::I64)?;
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
            .ins()
            .isub(lhs_cl, rhs_cl);
        let data = ValueData::from_cl(res, types::I64)?;
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
            .ins()
            .imul(lhs_cl, rhs_cl);
        let data = ValueData::from_cl(res, types::I64)?;
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
            .ins()
            .udiv(lhs_cl, rhs_cl);
        let data = ValueData::from_cl(res, types::I64)?;
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
            .ins()
            .band(lhs_cl, rhs_cl);
        let data = ValueData::from_cl(res, types::I64)?;
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
            .ins()
            .bor(lhs_cl, rhs_cl);
        let data = ValueData::from_cl(res, types::I64)?;
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
            .ins()
            .bxor(lhs_cl, rhs_cl);
        let data = ValueData::from_cl(res, types::I64)?;
        Ok(self.value_store.new_value(data))
    }

    pub fn cmp(&mut self, cmp_type: CondCode, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let cc = match cmp_type {
            CondCode::Equal => condcodes::IntCC::Equal,
            CondCode::NotEqual => condcodes::IntCC::NotEqual,
            CondCode::LessThan => condcodes::IntCC::SignedLessThan,
            CondCode::GreaterThanOrEqual => condcodes::IntCC::SignedGreaterThanOrEqual,
            CondCode::GreaterThan => condcodes::IntCC::SignedGreaterThan,
            CondCode::LessThanOrEqual => condcodes::IntCC::SignedLessThanOrEqual,
        };

        let lhs_cl = self.to_cl(lhs)?;
        let rhs_cl = self.to_cl(rhs)?;
        let res = self
            .inst_builder
            .ins()
            .icmp(cc, lhs_cl, rhs_cl);
        let data = ValueData::from_cl(res, types::B1)?;
        Ok(self.value_store.new_value(data))
    }

    pub fn set_var(&mut self, name: &str, val: Value) -> Result<Value, Error> {
        let variable = self.scope_stack.get_var(name).unwrap_or({
            let variable = Variable::new(self.scope_stack.variables().count());
            self.scope_stack.add(name, val, variable);
            self.inst_builder
                .declare_var(variable, val.get_type().cl_type()?);
            variable
        });
        if let Ok(val) = self.to_cl(val) {
            self.inst_builder.def_var(variable, val);
        }
        self.scope_stack.set(name, val);
        Ok(val)
    }

    pub fn get_var(&mut self, name: &str) -> Option<Value> {
        self.scope_stack.get_var(name).map(|var| {
            let value = self.scope_stack.get(name).unwrap();
            let data = ValueData::primitive(self.inst_builder.use_var(var), value.get_type());
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
        let cl = self.to_cl(v)?;
        Ok(match (v.get_type(), t) {
            (Type::Number, Type::Boolean) => {
                let zero = self.number_constant(0)?;
                self.cmp(CondCode::NotEqual, v, zero)?
            }
            (Type::Boolean, Type::Number) => {
                let data = ValueData::primitive(self.inst_builder.ins().bint(t.cl_type()?, cl), t);
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

    pub fn alloc(&mut self, size: u32) -> Result<Slot, Error> {
        let ss = self.inst_builder.create_stack_slot(stackslot::StackSlotData::new(stackslot::StackSlotKind::ExplicitSlot, size));
        Ok(Slot::new(ss, size))
    }

    pub fn store(&mut self, v: Value, slot: Slot, offset: i32) -> Result<(), Error> {
        let cl = self.to_cl(v)?;
        self.inst_builder.ins().stack_store(cl, slot.cl_slot(), Offset32::new(offset));
        Ok(())
    }

    pub fn load(&mut self, t: Type, slot: Slot, offset: i32) -> Result<Value, Error> {
        let data = ValueData::from_cl(self.inst_builder.ins().stack_load(t.cl_type()?, slot.cl_slot(), Offset32::new(offset)), t.cl_type()?)?;
        Ok(self.value_store.new_value(data))
    }

    pub fn create_block(&mut self) -> Block {
        let ebb = self.inst_builder.create_ebb();
        Block { ebb }
    }

    pub fn brz(&mut self, condition: Value, block: Block) -> Result<(), Error> {
        if condition.get_type() != Type::Boolean {
            return Err(TypeError.into());
        }
        let cl = self.to_cl(condition)?;
        self.inst_builder
            .ins()
            .brz(cl, block.cl_ebb(), &[]);
        Ok(())
    }

    pub fn set_block_signature(&mut self, block: Block, types: &[Type]) -> Result<(), Error> {
        for t in types {
            self.inst_builder
                .append_ebb_param(block.cl_ebb(), t.cl_type()?);
        }
        self.block_table.insert(block, types.to_vec());
        Ok(())
    }

    pub fn jump(&mut self, block: Block, args: &[Value]) {
        let cl_args: Vec<_> = args.into_iter().filter_map(|v| self.to_cl(*v).ok()).collect();
        self.inst_builder.ins().jump(block.cl_ebb(), &cl_args);
    }

    pub fn switch_to_block(&mut self, block: Block) {
        self.inst_builder.switch_to_block(block.cl_ebb());
        self.inst_builder.seal_block(block.cl_ebb());
    }

    pub fn block_params(&mut self, block: Block) -> Box<Vec<Value>> {
        let signature = self.block_table.get(&block).unwrap();
        let store = &mut self.value_store;
        let params: Vec<_> = self
            .inst_builder
            .ebb_params(block.cl_ebb())
            .into_iter()
            .zip(signature.into_iter())
            .map(|(v, t)| store.new_value(ValueData::primitive(*v, *t)))
            .collect();
        Box::new(params)
    }
}
