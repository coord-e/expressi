use value::{Value, Type};
use expression::Operator;

use cranelift::codegen::ir::{InstBuilder, InstBuilderBase, types, condcodes, entities};
use cranelift::prelude::{EntityRef, FunctionBuilder, Variable};

use std::collections::HashMap;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CondCode {
    Equal,
    NotEqual,
    LessThan,
    GreaterThanOrEqual,
    GreaterThan,
    LessThanOrEqual
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Block {
    ebb: entities::Ebb
}

impl Block {
    pub fn cl_ebb(&self) -> entities::Ebb {
        self.ebb
    }
}

pub struct Builder<'a> {
    pub inst_builder: FunctionBuilder<'a, Variable>,
    pub variable_map: HashMap<String, Variable>,
    pub variable_value_map: HashMap<usize, Value>,
    pub block_table: HashMap<Block, &'a [Type]>
}

impl<'a> Builder<'a> {
    pub fn inst_builder(&self) -> FunctionBuilder<'a, Variable> {
        self.inst_builder
    }

    pub fn finalize(&self) {
        self.inst_builder.finalize()
    }

    pub fn number_constant(&self, v: i64) -> Value {
        let t = types::I64;
        Value::new(self.inst_builder.ins().iconst(t, v), t)
    }

    pub fn boolean_constant(&self, v: bool) -> Value {
        let t = types::B1;
        Value::new(self.inst_builder.ins().bconst(t, v), t)
    }

    pub fn apply_op(&self, op: Operator, lhs: Value, rhs: Value) -> Value {
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

    pub fn add(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().iadd(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn sub(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().isub(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn mul(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().imul(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn div(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().udiv(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn bit_and(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().band(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn bit_or(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().bor(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn bit_xor(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().bxor(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn cmp(&self, cmp_type: CondCode, lhs: Value, rhs: Value) -> Value {
        let cc = match cmp_type {
            CondCode::Equal => condcodes::IntCC::Equal,
            CondCode::NotEqual => condcodes::IntCC::NotEqual,
            CondCode::LessThan => condcodes::IntCC::SignedLessThan,
            CondCode::GreaterThanOrEqual => condcodes::IntCC::SignedGreaterThanOrEqual,
            CondCode::GreaterThan => condcodes::IntCC::SignedGreaterThan,
            CondCode::LessThanOrEqual => condcodes::IntCC::SignedLessThanOrEqual
        };

        let res = self.inst_builder.ins().icmp(cc, lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn set_var(&self, name: &str, val: Value) {
        let variable = if self.variable_map.contains_key(name) {
            *self.variable_map.get(name).unwrap()
        } else {
            let variable = Variable::new(self.variable_map.len());
            self.variable_map.insert(name.to_owned(), variable);
            self.inst_builder.declare_var(variable, val.get_type().cl_type().unwrap());
            variable
        };
        self.inst_builder.def_var(variable, val.cl_value());
        self.variable_value_map.insert(variable.index(), val);
    }

    pub fn get_var(&self, name: &str) -> Option<Value> {
        if let Some(variable) = self.variable_map.get(name) {
            let value = self.variable_value_map.get(&variable.index()).unwrap();
            self.variable_map.get(&name.to_owned()).map(|var| Value { cranelift_value: self.inst_builder.use_var(*var), .. *value })
        } else {
            None
        }
    }

    pub fn create_block(&self) -> Block {
        let ebb = self.inst_builder.create_ebb();
        Block { ebb }
    }

    pub fn brz(&self, condition: Value, block: Block) {
        self.inst_builder.ins().brz(condition.cl_value(), block.cl_ebb(), &[]);
    }

    pub fn set_block_signature(&self, block: Block, types: &'a [Type]) {
        for t in types {
            self.inst_builder.append_ebb_param(block.cl_ebb(), t.cl_type().unwrap());
        }
        self.block_table.insert(block, types);
    }

    pub fn jump(&self, block: Block, args: &[Value]) {
        let cl_args: Vec<_> = args.into_iter().map(|v| v.cl_value()).collect();
        self.inst_builder.ins().jump(block.cl_ebb(), &cl_args);
    }

    pub fn switch_to_block(&self, block: Block) {
        self.inst_builder.switch_to_block(block.cl_ebb());
        self.inst_builder.seal_block(block.cl_ebb());
    }

    pub fn block_params(&self, block: Block) -> Box<Vec<Value>> {
        let signature = self.block_table.get(&block).unwrap();
        let params: Vec<_> = self.inst_builder.ebb_params(block.cl_ebb()).into_iter().zip(signature.into_iter()).map(|(v, t)| Value {cranelift_value: *v, value_type: *t}).collect();
        Box::new(params)
    }
}
