use error::{InvalidCastError, TypeError};
use expression::Operator;
use value::{Type, Value};
use scope::{Scope, ScopeStack};

use failure::Error;

use cranelift::codegen::ir::{condcodes, entities, types, InstBuilder};
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
    pub block_table: HashMap<Block, Vec<Type>>
}

impl<'a> Builder<'a> {
    pub fn inst_builder<'short>(&'short mut self) -> &'short mut FunctionBuilder<'a, Variable> {
        self.inst_builder
    }

    pub fn finalize(&mut self) {
        self.inst_builder.finalize()
    }

    pub fn number_constant(&mut self, v: i64) -> Result<Value, Error> {
        let t = types::I64;
        Value::new(self.inst_builder.ins().iconst(t, v), t)
    }

    pub fn boolean_constant(&mut self, v: bool) -> Result<Value, Error> {
        let t = types::B1;
        Value::new(self.inst_builder.ins().bconst(t, v), t)
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
        let res = self
            .inst_builder
            .ins()
            .iadd(lhs.cl_value()?, rhs.cl_value()?);
        Value::new(res, types::I64)
    }

    pub fn sub(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let res = self
            .inst_builder
            .ins()
            .isub(lhs.cl_value()?, rhs.cl_value()?);
        Value::new(res, types::I64)
    }

    pub fn mul(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let res = self
            .inst_builder
            .ins()
            .imul(lhs.cl_value()?, rhs.cl_value()?);
        Value::new(res, types::I64)
    }

    pub fn div(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let res = self
            .inst_builder
            .ins()
            .udiv(lhs.cl_value()?, rhs.cl_value()?);
        Value::new(res, types::I64)
    }

    pub fn bit_and(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let res = self
            .inst_builder
            .ins()
            .band(lhs.cl_value()?, rhs.cl_value()?);
        Value::new(res, types::I64)
    }

    pub fn bit_or(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let res = self
            .inst_builder
            .ins()
            .bor(lhs.cl_value()?, rhs.cl_value()?);
        Value::new(res, types::I64)
    }

    pub fn bit_xor(&mut self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        if lhs.get_type() != Type::Number || rhs.get_type() != Type::Number {
            return Err(TypeError.into());
        }
        let res = self
            .inst_builder
            .ins()
            .bxor(lhs.cl_value()?, rhs.cl_value()?);
        Value::new(res, types::I64)
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

        let res = self
            .inst_builder
            .ins()
            .icmp(cc, lhs.cl_value()?, rhs.cl_value()?);
        Value::new(res, types::B1)
    }

    pub fn set_var(&mut self, name: &str, val: Value) -> Result<Value, Error> {
        let variable = self.scope_stack.get_var(name).unwrap_or({
            let variable = Variable::new(self.scope_stack.variables().count());
            self.scope_stack.add(name, val, variable);
            self.inst_builder
                .declare_var(variable, val.get_type().cl_type()?);
            variable
        });
        if let Ok(val) = val.cl_value() {
            self.inst_builder.def_var(variable, val);
        }
        self.scope_stack.set(name, val);
        Ok(val)
    }

    pub fn get_var(&mut self, name: &str) -> Option<Value> {
        self.scope_stack.get_var(name).map(|var| {
            let value = self.scope_stack.get(name).unwrap();
            Value {
                cranelift_value: Some(self.inst_builder.use_var(var)),
                ..*value
            }
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
            (Type::Boolean, Type::Number) => Value {
                cranelift_value: Some(self.inst_builder.ins().bint(t.cl_type()?, v.cl_value()?)),
                value_type: t,
                ..v
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

    pub fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }

    pub fn create_block(&mut self) -> Block {
        let ebb = self.inst_builder.create_ebb();
        Block { ebb }
    }

    pub fn brz(&mut self, condition: Value, block: Block) -> Result<(), Error> {
        if condition.get_type() != Type::Boolean {
            return Err(TypeError.into());
        }
        self.inst_builder
            .ins()
            .brz(condition.cl_value()?, block.cl_ebb(), &[]);
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
        let cl_args: Vec<_> = args.into_iter().filter_map(|v| v.cl_value().ok()).collect();
        self.inst_builder.ins().jump(block.cl_ebb(), &cl_args);
    }

    pub fn switch_to_block(&mut self, block: Block) {
        self.inst_builder.switch_to_block(block.cl_ebb());
        self.inst_builder.seal_block(block.cl_ebb());
    }

    pub fn block_params(&self, block: Block) -> Box<Vec<Value>> {
        let signature = self.block_table.get(&block).unwrap();
        let params: Vec<_> = self
            .inst_builder
            .ebb_params(block.cl_ebb())
            .into_iter()
            .zip(signature.into_iter())
            .map(|(v, t)| Value {
                cranelift_value: Some(*v),
                value_type: *t,
            })
            .collect();
        Box::new(params)
    }
}
