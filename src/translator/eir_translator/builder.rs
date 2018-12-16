use error::TranslationError;
use expression::Operator;
use ir::{self, BindingKind};
use scope::{Env, Scope, ScopedEnv};
use transform::type_infer::Type;
use translator::eir_translator::atom::Atom;
use translator::eir_translator::BoundPointer;

use failure::Error;

use inkwell::types::BasicType;
use inkwell::{basic_block, builder, module, types, values, AddressSpace, IntPredicate};

use std::collections::{BTreeMap, HashMap};
use std::mem;
use std::rc::Rc;

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
    env: ScopedEnv<BoundPointer>,
}

impl<'a> Builder<'a> {
    pub fn new(inst_builder: &'a mut builder::Builder, module: Rc<module::Module>) -> Self {
        Builder {
            inst_builder,
            module,
            env: ScopedEnv::new(),
        }
    }

    pub fn inst_builder<'short>(&'short mut self) -> &'short mut builder::Builder {
        self.inst_builder
    }

    pub fn env<'short>(&'short mut self) -> &'short mut ScopedEnv<BoundPointer> {
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

    pub fn llvm_type(&self, ty: &Type) -> Result<types::BasicTypeEnum, Error> {
        Ok(match ty {
            Type::Number => types::IntType::i64_type().into(),
            Type::Boolean => types::IntType::bool_type().into(),
            Type::Empty => types::VoidType::void_type()
                .ptr_type(AddressSpace::Generic)
                .into(),
            Type::Variable(_) => return Err(TranslationError::UnresolvedType.into()),
            Type::Function(box param, box body) => {
                let param = self.llvm_type(param)?;
                let ret = self.llvm_type(body)?;
                // Capture list
                let void_ptr_ty = types::VoidType::void_type().ptr_type(AddressSpace::Generic);
                ret.fn_type(&[param, void_ptr_ty.into()], false)
                    .ptr_type(AddressSpace::Generic)
                    .into()
            }
        })
    }

    pub fn number_constant(&mut self, v: i64) -> Result<values::BasicValueEnum, Error> {
        let t = types::IntType::i64_type();
        Ok(values::BasicValueEnum::IntValue(
            t.const_int(v.abs() as u64, v < 0),
        ))
    }

    pub fn boolean_constant(&mut self, v: bool) -> Result<values::BasicValueEnum, Error> {
        let t = types::IntType::bool_type();
        Ok(values::BasicValueEnum::IntValue(
            t.const_int(v as u64, false),
        ))
    }

    pub fn empty_constant(&self) -> Result<values::BasicValueEnum, Error> {
        let t = types::VoidType::void_type().ptr_type(AddressSpace::Generic);
        Ok(values::BasicValueEnum::PointerValue(t.const_null()))
    }

    pub fn function_constant(
        &mut self,
        ty: &Type,
        param_name: String,
        capture_list: &BTreeMap<String, Type>,
    ) -> Result<values::BasicValueEnum, Error> {
        let fn_type = self
            .llvm_type(ty)?
            .into_pointer_type()
            .get_element_type()
            .into_function_type();

        let function = self.module.add_function("", fn_type, None);
        let basic_block = self
            .module
            .get_context()
            .append_basic_block(&function, "entry");
        self.inst_builder.position_at_end(&basic_block);
        let arg_ptr = self
            .inst_builder
            .build_alloca(fn_type.get_param_types()[0], "");
        self.inst_builder
            .build_store(arg_ptr, function.get_first_param().unwrap());
        self.env.insert(
            &param_name,
            BoundPointer::new(BindingKind::Immutable, arg_ptr.into()),
        );

        let captures_ptr = function.get_nth_param(1).unwrap();
        let captures_type = self.capture_list_type(capture_list)?;
        let captures_ptr_typed = self.inst_builder.build_pointer_cast(
            captures_ptr.into_pointer_value(),
            captures_type.ptr_type(AddressSpace::Generic),
            "captures",
        );
        for (i, (name, _)) in capture_list.iter().enumerate() {
            let elem_ptr = unsafe {
                self.inst_builder
                    .build_struct_gep(captures_ptr_typed, i as u32, "")
            };
            self.env.insert(
                &name,
                BoundPointer::new(BindingKind::Immutable, elem_ptr.into()),
            );
        }

        let ptr: values::PointerValue = unsafe { mem::transmute(function) };
        Ok(ptr.into())
    }

    pub fn capture_list_type(
        &mut self,
        list: &BTreeMap<String, Type>,
    ) -> Result<types::StructType, Error> {
        let types: Vec<_> = list
            .iter()
            .map(|(_, v)| self.llvm_type(v))
            .collect::<Result<Vec<types::BasicTypeEnum>, Error>>()?;
        let struct_type = types::StructType::struct_type(&types, false);
        Ok(struct_type)
    }

    pub fn extract_func(
        &self,
        func: Atom<values::BasicValueEnum>,
        func_ty: &Type,
    ) -> values::BasicValueEnum {
        match func {
            Atom::LLVMValue(func) => func,
            Atom::PolyValue(func_table) => *func_table.get(func_ty).unwrap(),
            Atom::CapturingValue(box func_v, _) => self.extract_func(func_v, func_ty),
        }
    }

    pub fn call(
        &mut self,
        func: values::BasicValueEnum,
        arg: values::BasicValueEnum,
        capture_list: Option<(Vec<values::BasicValueEnum>, types::StructType)>,
    ) -> Result<values::BasicValueEnum, Error> {
        let void_ptr_ty = types::VoidType::void_type().ptr_type(AddressSpace::Generic);
        let capture_ptr = if let Some((captures, struct_type)) = capture_list {
            let typed_ptr = self.inst_builder.build_alloca(struct_type, "capture_struct");
            for (i, v) in captures.into_iter().enumerate() {
                let elem_ptr = unsafe { self.inst_builder.build_struct_gep(typed_ptr, i as u32, "") };
                self.inst_builder.build_store(elem_ptr, v);
            }
            self.inst_builder.build_pointer_cast(typed_ptr, void_ptr_ty, "")
        } else {
            void_ptr_ty.const_null() // nullptr for empty captures
        };
        let func_ptr = func.into_pointer_value();
        let func_v: values::FunctionValue = unsafe { mem::transmute(func_ptr) };
        let call_inst = self
            .inst_builder
            .build_call(func_v, &[arg, capture_ptr.into()], "");
        Ok(call_inst.try_as_basic_value().left().unwrap().into())
    }

    pub fn apply_op(
        &mut self,
        op: Operator,
        lhs: values::BasicValueEnum,
        rhs: values::BasicValueEnum,
    ) -> Result<values::BasicValueEnum, Error> {
        let lhs_int = lhs.into_int_value();
        let rhs_int = rhs.into_int_value();
        Ok(match op {
            Operator::Add => self.inst_builder.build_int_add(lhs_int, rhs_int, "add"),
            Operator::Sub => self.inst_builder.build_int_sub(lhs_int, rhs_int, "sub"),
            Operator::Mul => self.inst_builder.build_int_mul(lhs_int, rhs_int, "mul"),
            Operator::Div => self
                .inst_builder
                .build_int_unsigned_div(lhs_int, rhs_int, "div"),
            Operator::BitAnd => self.inst_builder.build_and(lhs_int, rhs_int, "add"),
            Operator::BitXor => self.inst_builder.build_xor(lhs_int, rhs_int, "xor"),
            Operator::BitOr => self.inst_builder.build_or(lhs_int, rhs_int, "or"),
            Operator::Lt => self.cmp(CondCode::LessThan, lhs_int, rhs_int),
            Operator::Gt => self.cmp(CondCode::GreaterThan, lhs_int, rhs_int),
            Operator::Le => self.cmp(CondCode::LessThanOrEqual, lhs_int, rhs_int),
            Operator::Ge => self.cmp(CondCode::GreaterThanOrEqual, lhs_int, rhs_int),
            Operator::Eq => self.cmp(CondCode::Equal, lhs_int, rhs_int),
            Operator::Ne => self.cmp(CondCode::NotEqual, lhs_int, rhs_int),
            Operator::Index => self.index(lhs, rhs),
        }
        .into())
    }

    pub fn cmp(
        &mut self,
        cmp_type: CondCode,
        lhs: values::IntValue,
        rhs: values::IntValue,
    ) -> values::IntValue {
        let cc = match cmp_type {
            CondCode::Equal => IntPredicate::EQ,
            CondCode::NotEqual => IntPredicate::NE,
            CondCode::LessThan => IntPredicate::SLT,
            CondCode::GreaterThanOrEqual => IntPredicate::SGE,
            CondCode::GreaterThan => IntPredicate::SGT,
            CondCode::LessThanOrEqual => IntPredicate::SLE,
        };

        self.inst_builder.build_int_compare(cc, lhs, rhs, "cmp")
    }

    pub fn index(
        &mut self,
        _lhs: values::BasicValueEnum,
        _rhs: values::BasicValueEnum,
    ) -> values::IntValue {
        unimplemented!()
    }

    pub fn alloca_atom(
        &mut self,
        val: &Atom<values::BasicValueEnum>,
    ) -> Atom<values::PointerValue> {
        match val {
            Atom::LLVMValue(val) => {
                let t = self.type_of(*val);
                self.inst_builder.build_alloca(t, "").into()
            }
            Atom::PolyValue(val_table) => val_table
                .iter()
                .map(|(k, v)| {
                    let t = self.type_of(*v);
                    (k.clone(), self.inst_builder.build_alloca(t, "").into())
                })
                .collect::<HashMap<_, _>>()
                .into(),
            Atom::CapturingValue(box val, capture_list) => {
                Atom::CapturingValue(box self.alloca_atom(val), capture_list.clone())
            }
        }
    }

    pub(crate) fn declare_mut_var(
        &mut self,
        name: &str,
        base_value: &Atom<values::BasicValueEnum>,
        unique: bool,
    ) -> Result<String, Error> {
        let real_name = if unique {
            self.env.unique_name(name)
        } else {
            name.to_string()
        };
        let ptr = self.alloca_atom(base_value);
        self.env
            .insert(&real_name, BoundPointer::new(BindingKind::Mutable, ptr));
        Ok(real_name)
    }

    pub fn bind_var(
        &mut self,
        name: &str,
        val: &Atom<values::BasicValueEnum>,
        kind: BindingKind,
    ) -> Result<(), Error> {
        let ptr = self.store_atom(name, val);
        self.env.insert(name, BoundPointer::new(kind, ptr));
        Ok(())
    }

    fn store_atom(
        &mut self,
        name: &str,
        val: &Atom<values::BasicValueEnum>,
    ) -> Atom<values::PointerValue> {
        match val {
            Atom::LLVMValue(val) => self.store_mono_var(name, *val).into(),
            Atom::PolyValue(val_table) => val_table
                .iter()
                .map(|(k, v)| (k.clone(), self.store_mono_var(name, *v)))
                .collect::<HashMap<_, _>>()
                .into(),
            Atom::CapturingValue(box v, capture_list) => {
                Atom::CapturingValue(box self.store_atom(name, v), capture_list.clone())
            }
        }
    }

    fn store_mono_var(&mut self, name: &str, val: values::BasicValueEnum) -> values::PointerValue {
        let llvm_type = self.type_of(val);
        let variable = self.inst_builder.build_alloca(llvm_type, name);
        self.inst_builder.build_store(variable, val);
        variable
    }

    fn store_ptr_atom(&mut self, val: &Atom<values::PointerValue>) -> Result<(), Error> {
        match val {
            Atom::LLVMValue(var) => {
                self.inst_builder
                    .build_store(*var, val.clone().expect_value()?);
            }
            Atom::PolyValue(var_table) => {
                var_table
                    .iter()
                    .map(|(k, v)| {
                        self.inst_builder
                            .build_store(*v, *val.clone().expect_poly_value()?.get(k).unwrap());
                        Ok(())
                    })
                    .collect::<Result<(), Error>>()?;
            }
            Atom::CapturingValue(box v, capture_list) => self.store_ptr_atom(v)?,
        }
        Ok(())
    }

    pub fn assign_var(
        &mut self,
        name: &str,
        val: &Atom<values::BasicValueEnum>,
    ) -> Result<(), Error> {
        let var = self
            .env
            .get(name)
            .ok_or(TranslationError::UndeclaredVariable)?;

        if var.kind() != BindingKind::Mutable {
            return Err(TranslationError::ImmutableAssign.into());
        }

        self.store_ptr_atom(var.ptr_value())
    }

    fn load_ptr_atom(&self, val: &Atom<values::PointerValue>) -> Atom<values::BasicValueEnum> {
        match val {
            Atom::LLVMValue(var) => self.inst_builder.build_load(var.clone(), "load_var").into(),
            Atom::PolyValue(var_table) => var_table
                .into_iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        self.inst_builder.build_load(v.clone(), "load_var"),
                    )
                })
                .collect::<HashMap<_, _>>()
                .into(),
            Atom::CapturingValue(box v, captures_list) => {
                Atom::CapturingValue(box self.load_ptr_atom(v), captures_list.clone())
            }
        }
    }

    pub fn get_var(&mut self, name: &str) -> Result<Option<Atom<values::BasicValueEnum>>, Error> {
        self.env.get(name).map_or(Ok(None), |var| {
            Ok(Some(self.load_ptr_atom(var.ptr_value())))
        })
    }

    pub fn cast_to(
        &mut self,
        v: values::BasicValueEnum,
        to_type: types::BasicTypeEnum,
    ) -> Result<values::BasicValueEnum, Error> {
        let from_type = self.type_of(v);
        if from_type == to_type {
            return Err(TranslationError::InvalidCast {
                from: format!("{:?}", from_type),
                to: format!("{:?}", to_type),
            }
            .into());
        }

        let number_type: types::BasicTypeEnum = types::IntType::i64_type().into();
        let bool_type: types::BasicTypeEnum = types::IntType::bool_type().into();

        // TODO: more elegant way to match types
        if from_type == number_type {
            if to_type == bool_type {
                let zero = self.number_constant(0)?;
                return Ok(self
                    .cmp(
                        CondCode::NotEqual,
                        v.into_int_value(),
                        zero.into_int_value(),
                    )
                    .into());
            }
        } else if from_type == bool_type {
            if to_type == number_type {
                return Ok(self
                    .inst_builder
                    .build_int_z_extend(v.into_int_value(), to_type.into_int_type(), "b2i")
                    .into());
            }
        }
        Err(TranslationError::InvalidCast {
            from: format!("{:?}", from_type),
            to: format!("{:?}", to_type),
        }
        .into())
    }

    pub fn enter_new_scope(&mut self) {
        let scope = self.env.new_scope();
        self.enter_scope(scope);
    }

    pub fn enter_scope(&mut self, sc: Env<BoundPointer>) {
        self.env.push(sc);
    }

    pub fn exit_scope(&mut self) -> Result<Env<BoundPointer>, Error> {
        self.env.pop()
    }

    pub fn array_alloc(
        &mut self,
        _t: types::BasicTypeEnum,
        _size: u32,
    ) -> Result<values::PointerValue, Error> {
        unimplemented!()
    }

    pub fn store(
        &mut self,
        _v: values::BasicValueEnum,
        _addr: values::PointerValue,
        _offset: u32,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn load(
        &mut self,
        _t: &Type,
        _addr: values::PointerValue,
        _offset: u32,
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
        let bool_type = types::IntType::bool_type();
        if self.type_of(condition) != bool_type.into() {
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
        let number_type: types::BasicTypeEnum = types::IntType::i64_type().into();
        let return_value: values::BasicValueEnum = if self.type_of(v) != number_type {
            self.cast_to(v, number_type)?
        } else {
            v
        };
        // Emit the return instruction.
        self.inst_builder.build_return(Some(&return_value));
        Ok(())
    }
}
