use error::{InternalError, TranslationError};
use ir;
use transform::type_infer::Type;
use translator::eir_translator::{Atom, Builder};

use failure::Error;
use inkwell::values::BasicValueEnum;
use std::collections::{BTreeMap, HashMap, HashSet};

pub struct EIRTranslator<'a> {
    pub builder: Builder<'a>,
}

impl<'a> EIRTranslator<'a> {
    fn translate_monotype_function(
        &mut self,
        param: ir::Identifier,
        ty: &Type,
        body: ir::Value,
    ) -> Result<BasicValueEnum, Error> {
        let previous_block = self.builder.inst_builder().get_insert_block().unwrap();
        self.builder.enter_new_scope();
        let function = self.builder.function_constant(&ty, param)?;
        let ret = self.translate_expr(body)?.expect_value()?;
        self.builder.exit_scope()?;
        self.builder.inst_builder().build_return(Some(&ret));
        self.builder.inst_builder().position_at_end(&previous_block);
        Ok(function)
    }

    pub fn translate_expr(&mut self, expr: ir::Value) -> Result<Atom<BasicValueEnum>, Error> {
        Ok(match expr {
            ir::Value::Typed(ty, ty_candidates, box value) => match value {
                ir::Value::Constant(c) => match c {
                    ir::Constant::Number(number) => {
                        self.builder.number_constant(i64::from(number))?.into()
                    }
                    ir::Constant::Boolean(tf) => self.builder.boolean_constant(tf)?.into(),
                    ir::Constant::Empty => self.builder.empty_constant()?.into(),
                },
                ir::Value::Function(param, box body, capture_list) => {
                    if ty_candidates.is_empty() {
                        self.translate_monotype_function(param, &ty, body)?.into()
                    } else {
                        ty_candidates
                            .into_iter()
                            .map(|(ty, body)| {
                                match body {
                                    ir::Value::Function(param, box body, _) => {
                                        self.translate_monotype_function(param, &ty, body)
                                    }
                                    _ => unreachable!(),
                                }
                                .map(|v| (ty, v))
                            })
                            .collect::<Result<HashMap<_, _>, _>>()?
                            .into()
                    }
                }
                ir::Value::Typed(..) => bail!(InternalError::DoubleTyped),
                _ => self.translate_expr(value)?.into(),
            },
            ir::Value::Apply(box func, box arg) => {
                let func_ty = func.type_().ok_or(TranslationError::NotTyped)?;
                let func = self.translate_expr(func.clone())?;
                let arg = self.translate_expr(arg)?.expect_value()?;
                match func {
                    Atom::LLVMValue(func) => self.builder.call(func, arg)?.into(),
                    Atom::PolyValue(func_table) => self
                        .builder
                        .call(*func_table.get(func_ty).unwrap(), arg)?
                        .into(),
                }
            }
            ir::Value::BinOp(op, lhs, rhs) => {
                let lhs = self.translate_expr(*lhs)?.expect_value()?;
                let rhs = self.translate_expr(*rhs)?.expect_value()?;
                self.builder.apply_op(op, lhs, rhs)?.into()
            }

            ir::Value::Follow(lhs, rhs) => {
                self.translate_expr(*lhs)?;
                self.translate_expr(*rhs)?
            }

            ir::Value::Bind(kind, name, rhs) => {
                let new_value = self.translate_expr(*rhs)?;
                self.builder.bind_var(&name, &new_value, kind)?;
                new_value.into()
            }

            ir::Value::Assign(lhs, rhs) => {
                let new_value = self.translate_expr(*rhs)?;
                let name = match *lhs {
                    ir::Value::Typed(_, _, box ir::Value::Variable(name)) => name,
                    _ => panic!("Non-variable identifier"),
                };
                self.builder.assign_var(&name, &new_value)?;
                new_value
            }

            ir::Value::Variable(name) => self
                .builder
                .get_var(&name)
                .and_then(|v| v.ok_or(TranslationError::UndeclaredVariable.into()))?,

            ir::Value::Scope(expr) => {
                self.builder.enter_new_scope();
                let content = self.translate_expr(*expr)?;
                self.builder.exit_scope()?;
                content.into()
            }

            ir::Value::IfElse(cond, then_expr, else_expr) => {
                let condition_value = self.translate_expr(*cond)?.expect_value()?;

                let then_block = self.builder.create_block()?;
                let else_block = self.builder.create_block()?;
                let merge_block = self.builder.create_block()?;

                let initial_block = self.builder.current_block()?;

                self.builder.switch_to_block(&then_block);
                let then_return = self.translate_expr(*then_expr)?;

                self.builder.switch_to_block(&initial_block);
                let var_name = self.builder.declare_mut_var("__cond", &then_return, true)?;
                self.builder
                    .brz(condition_value, &then_block, &else_block)?;

                self.builder.switch_to_block(&then_block);
                self.builder.assign_var(&var_name, &then_return)?;
                self.builder.jump(&merge_block);

                // Start writing 'else' block
                self.builder.switch_to_block(&else_block);
                let else_return = self.translate_expr(*else_expr)?;
                self.builder.assign_var(&var_name, &else_return)?;

                // Jump to merge block after translation of the 'then' block
                self.builder.jump(&merge_block);

                self.builder.switch_to_block(&merge_block);
                self.builder.get_var(&var_name)?.unwrap().into()
            }
            _ => bail!(TranslationError::NotTyped),
        })
    }
}
