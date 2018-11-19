use error::TranslationError;
use expression::Operator;
use scope::{Scope, ScopedEnv, Env};
use value::manager::PrimitiveKind;
use value::type_::EnumTypeData;
use value::{TypeID, TypeStore, ValueData, ValueManager, ValueManagerRef, ValueStore};
use expression::Expression;

use failure::Error;

use inkwell::{basic_block, builder, module, types, values, IntPredicate, AddressSpace};

use std::rc::Rc;

#[derive(PartialEq, Debug, Clone, Eq)]
pub enum BindingKind {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CondCode {
    Equal,
    NotEqual,
    LessThan,
    GreaterThanOrEqual,
    GreaterThan,
    LessThanOrEqual,
}

pub struct Block {
    ebb: basic_block::BasicBlock,
}

impl Block {
    pub fn cl_ebb(&self) -> &basic_block::BasicBlock {
        &self.ebb
    }
}

pub struct Builder<'a> {
    inst_builder: &'a mut builder::Builder,
    module: Rc<module::Module>,
    type_store: &'a mut TypeStore,
    env: ScopedEnv<values::PointerValue>,
}

impl<'a> Builder<'a> {
    pub fn new(
        type_store: &'a mut TypeStore,
        inst_builder: &'a mut builder::Builder,
        module: Rc<module::Module>,
    ) -> Self {
        Builder {
            inst_builder,
            module,
            type_store,
            env: ScopedEnv::new(),
        }
    }

    pub fn inst_builder<'short>(&'short mut self) -> &'short mut builder::Builder {
        self.inst_builder
    }

    pub fn env<'short>(&'short mut self) -> &'short mut ScopedEnv<values::PointerValue> {
        &mut self.env
    }

    pub fn type_of(&self, v: values::BasicValueEnum) -> types::BasicTypeEnum {
        match v {
            values::BasicValueEnum::ArrayValue(v) => v.get_type().into(),
            values::BasicValueEnum::IntValue(v) => v.get_type().into(),
            values::BasicValueEnum::FloatValue(v) => v.get_type().into(),
            values::BasicValueEnum::PointerValue(v) => v.get_type().into(),
            values::BasicValueEnum::StructValue(v) => v.get_type().into(),
            values::BasicValueEnum::VectorValue(v) => v.get_type().into(),
        }
    }

    pub fn number_constant(&mut self, v: i64) -> Result<values::BasicValueEnum, Error> {
        let t = types::IntType::i64_type();
        values::BasicValueEnum::IntValue(t.const_int(v.abs() as u64, v < 0))
    }

    pub fn boolean_constant(&mut self, v: bool) -> Result<values::BasicValueEnum, Error> {
        let t = types::IntType::bool_type();
        values::BasicValueEnum::IntValue(t.const_int(v as u64, false))
    }

    pub fn empty_constant(&self) -> Result<values::BasicValueEnum, Error> {
        let t = types::VoidType::void_type().ptr_type(AddressSpace::Generic);
        values::BasicValueEnum::PointerValue(t.const_null())
    }

    pub fn register_type(&mut self, data: EnumTypeData) -> Result<TypeID, Error> {
        self.type_store.new_enum(data)
    }

    pub fn apply_op(&mut self, op: Operator, lhs: values::BasicValueEnum, rhs: values::BasicValueEnum) -> Result<values::BasicValueEnum, Error> {
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

    pub fn add(&mut self, lhs: values::BasicValueEnum, rhs: values::BasicValueEnum) -> Result<values::BasicValueEnum, Error> {
        self.inst_builder.build_int_add(
            lhs.into_int_value(),
            rhs.into_int_value(),
            "add",
        )
    }

    pub fn sub(&mut self, lhs: values::BasicValueEnum, rhs: values::BasicValueEnum) -> Result<values::BasicValueEnum, Error> {
        self.inst_builder.build_int_sub(
            lhs.into_int_value(),
            rhs.into_int_value(),
            "sub",
        )
    }

    pub fn mul(&mut self, lhs: values::BasicValueEnum, rhs: values::BasicValueEnum) -> Result<values::BasicValueEnum, Error> {
        self.inst_builder.build_int_mul(
            lhs.into_int_value(),
            rhs.into_int_value(),
            "mul",
        )
    }

    pub fn div(&mut self, lhs: values::BasicValueEnum, rhs: values::BasicValueEnum) -> Result<values::BasicValueEnum, Error> {
        self.inst_builder.build_int_unsigned_div(
            lhs.into_int_value(),
            rhs.into_int_value(),
            "div",
        )
    }

    pub fn bit_and(&mut self, lhs: values::BasicValueEnum, rhs: values::BasicValueEnum) -> Result<values::BasicValueEnum, Error> {
        self.inst_builder
            .build_and(lhs.into_int_value(), rhs.into_int_value(), "and")
    }

    pub fn bit_or(&mut self, lhs: values::BasicValueEnum, rhs: values::BasicValueEnum) -> Result<values::BasicValueEnum, Error> {
        self.inst_builder
            .build_or(lhs.into_int_value(), rhs.into_int_value(), "or")
    }

    pub fn bit_xor(&mut self, lhs: values::BasicValueEnum, rhs: values::BasicValueEnum) -> Result<values::BasicValueEnum, Error> {
        self.inst_builder
            .build_xor(lhs.into_int_value(), rhs.into_int_value(), "xor")
    }

    pub fn cmp(
        &mut self,
        cmp_type: CondCode,
        lhs: values::BasicValueEnum,
        rhs: values::BasicValueEnum,
    ) -> Result<values::BasicValueEnum, Error> {
        let cc = match cmp_type {
            CondCode::Equal => IntPredicate::EQ,
            CondCode::NotEqual => IntPredicate::NE,
            CondCode::LessThan => IntPredicate::SLT,
            CondCode::GreaterThanOrEqual => IntPredicate::SGE,
            CondCode::GreaterThan => IntPredicate::SGT,
            CondCode::LessThanOrEqual => IntPredicate::SLE,
        };

        self.inst_builder.build_int_compare(
            cc,
            lhs.into_int_value(),
            rhs.into_int_value(),
            "cmp",
        )
    }

    pub fn index(&mut self, lhs: values::BasicValueEnum, rhs: values::BasicValueEnum) -> Result<values::BasicValueEnum, Error> {
        unimplemented!()
    }

    pub fn declare_mut_var(
        &mut self,
        name: &str,
        t: types::BasicTypeEnum,
        unique: bool,
    ) -> Result<String, Error> {
        let real_name = if unique {
            self.scope_stack.unique_name(name)
        } else {
            name.to_string()
        };
        let variable = self.inst_builder.build_alloca(t, &real_name);
        self.env.insert(real_name, variable);
        Ok(real_name)
    }

    pub fn bind_var(
        &mut self,
        name: &str,
        val: values::BasicValueEnum,
        kind: BindingKind,
    ) -> Result<values::BasicValueEnum, Error> {
        let llvm_type = self.type_of(val);
        let variable = self.inst_builder.build_alloca(llvm_type, name);
        self.env.insert(name, variable);
        self.inst_builder.build_store(variable, val);
        Ok(val)
    }

    pub fn assign_var(&mut self, name: &str, val: values::BasicValueEnum) -> Result<values::BasicValueEnum, Error> {
        let var = self
            .env
            .get(name)
            .ok_or(TranslationError::UndeclaredVariable)?;
        self.inst_builder.build_store(var, val);
        Ok(val)
    }

    pub fn get_var(&mut self, name: &str) -> Result<Option<values::BasicValueEnum>, Error> {
        self.env.get(name).map_or(Ok(None), |var| {
            let loaded = self.inst_builder.build_load(var, "load_var");
            Ok(Some(loaded))
        })
    }

    pub fn cast_to(&mut self, v: values::BasicValueEnum, to_type: types::BasicTypeEnum) -> Result<values::BasicValueEnum, Error> {
        let from_type = self.type_of(v);
        if from_type == to_type {
            return Err(TranslationError::InvalidCast {
                from: from_type,
                to: to_type,
            }.into());
        }

        let number_type = types::IntType::i64_type();
        let bool_type = types::IntType::bool_type();

        // TODO: more elegant way to match types
        if from_type == number_type {
            if to_type == bool_type {
                let zero = self.number_constant(0)?;
                return self.cmp(CondCode::NotEqual, v, zero);
            }
        } else if from_type == bool_type {
            if to_type == number_type {
                return self.inst_builder.build_int_z_extend(
                        v.into_int_value(),
                        to_type.into_int_type(),
                        "b2i",
                );
            }
        }
        Err(TranslationError::InvalidCast {
            from: from_type,
            to: to_type,
        }.into())
    }

    pub fn enter_new_scope(&mut self) {
        let scope = self.env.new_scope();
        self.enter_scope(scope);
    }

    pub fn enter_scope(&mut self, sc: Env<values::PointerValue>) {
        self.env.push(sc);
    }

    pub fn exit_scope(&mut self) -> Result<Env<values::PointerValue>, Error> {
        self.env.pop()
    }

    pub fn array_alloc(&mut self, t: types::BasicTypeEnum, size: u32) -> Result<values::PointerValue, Error> {
        unimplemented!()
    }

    pub fn store(
        &mut self,
        v: values::BasicValueEnum,
        addr: values::PointerValue,
        offset: u32,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn load(
        &mut self,
        t: TypeID,
        addr: values::PointerValue,
        offset: u32,
    ) -> Result<values::BasicValueEnum, Error> {
        unimplemented!()
    }

    pub fn create_block(&mut self) -> Result<Block, Error> {
        let parent = self
            .inst_builder
            .get_insert_block()
            .and_then(|b| b.get_parent())
            .ok_or(TranslationError::InvalidContextBranch)?;
        let block = self.module.get_context().append_basic_block(&parent, "");
        Ok(Block { ebb: block })
    }

    pub fn brz(
        &mut self,
        condition: values::BasicValueEnum,
        then_block: &Block,
        else_block: &Block,
    ) -> Result<(), Error> {
        let bool_type = types::IntType::bool_type;
        if self.type_of(condition) != bool_type {
            return Err(TranslationError::InvalidType.into());
        }
        self.inst_builder.build_conditional_branch(
            condition.into_int_value(),
            then_block.cl_ebb(),
            else_block.cl_ebb(),
        );
        Ok(())
    }

    pub fn jump(&mut self, block: &Block) {
        self.inst_builder.build_unconditional_branch(block.cl_ebb());
    }

    pub fn switch_to_block(&mut self, block: &Block) {
        self.inst_builder.position_at_end(block.cl_ebb());
    }

    pub fn current_block(&self) -> Result<Block, Error> {
        self.inst_builder
            .get_insert_block()
            .ok_or(TranslationError::InvalidContextBranch.into())
            .map(|ebb| Block { ebb })
    }

    pub fn ret_int(&mut self, v: values::BasicValueEnum) -> Result<(), Error> {
        // TODO: Generic return
        let number_type = types::IntType::i64_type;
        let return_value = if self.type_of(v) != number_type {
            self.cast_to(v, number_type)?
        } else {
            v
        };
        // Emit the return instruction.
        let cl = return_value.into_int_value();
        self.inst_builder.build_return(Some(&cl));
        Ok(())
    }
}
