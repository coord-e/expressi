use crate::error::{InternalError, TranslationError};
use crate::ir;
use crate::transform::type_infer::Type;
use crate::translator::eir_translator::{Atom, Builder};

use failure::Error;
use inkwell::values::BasicValueEnum;
use std::collections::HashMap;

fn translate_monotype_function<'a>(
    builder: &mut Builder<'a>,
    param: String,
    ty: &Type,
    body: ir::Value,
    capture_list: &HashMap<ir::Identifier, Type>
) -> Result<BasicValueEnum, Error> {
    let function = builder.function_constant(&ty, param, capture_list, |builder| {
        translate_eir(builder, body)?.expect_value()
    })?;
    Ok(function)
}

pub fn translate_eir<'a>(builder: &mut Builder<'a>, expr: ir::Value) -> Result<Atom<BasicValueEnum>, Error> {
    Ok(match expr {
        ir::Value::Typed(ty, ty_candidates, box value) => match value {
            ir::Value::Constant(c) => match c {
                ir::Constant::Number(number) => {
                    builder.number_constant(i64::from(number))?.into()
                }
                ir::Constant::Boolean(tf) => builder.boolean_constant(tf)?.into(),
                ir::Constant::Empty => builder.empty_constant()?.into(),
            },
            ir::Value::Function(param, box body, capture_list) => {
                // TODO: Add more sufficient implementation to check whether PolyValue is needed or not
                if ty_candidates.len() <= 1 {
                    translate_monotype_function(builder, param, &ty, body, &capture_list)?.into()
                } else {
                    ty_candidates
                        .into_iter()
                        .map(|(ty, body)| {
                            match body {
                                ir::Value::Function(_, box body, _) => {
                                    translate_monotype_function(builder, param.clone(), &ty, body, &capture_list)
                                }
                                _ => unreachable!(),
                            }.map(|v| (ty, v))
                        }).collect::<Result<HashMap<_, _>, _>>()?
                        .into()
                }
            }
            ir::Value::Typed(..) => bail!(InternalError::DoubleTyped),
            _ => translate_eir(builder, value)?.into(),
        },
        ir::Value::Apply(box func, box arg) => {
            let func_ty = func.type_().ok_or(TranslationError::NotTyped)?;
            let func = translate_eir(builder, func.clone())?;
            let arg = translate_eir(builder, arg)?.expect_value()?;
            match func {
                Atom::LLVMValue(func) => builder.call(func, arg)?.into(),
                Atom::PolyValue(func_table) => builder
                    .call(*func_table.get(func_ty).unwrap(), arg)?
                    .into(),
            }
        }
        ir::Value::BinOp(op, lhs, rhs) => {
            let lhs = translate_eir(builder, *lhs)?.expect_value()?;
            let rhs = translate_eir(builder, *rhs)?.expect_value()?;
            builder.apply_op(op, lhs, rhs)?.into()
        }

        ir::Value::Follow(lhs, rhs) => {
            translate_eir(builder, *lhs)?;
            translate_eir(builder, *rhs)?
        }

        ir::Value::Bind(kind, name, rhs) => {
            let new_value = translate_eir(builder, *rhs)?;
            builder.bind_var(&name, &new_value, kind)?;
            new_value.into()
        }

        ir::Value::Assign(lhs, rhs) => {
            let new_value = translate_eir(builder, *rhs)?;
            let name = match *lhs {
                ir::Value::Typed(_, _, box ir::Value::Variable(name)) => name,
                _ => panic!("Non-variable identifier"),
            };
            builder.assign_var(&name, &new_value)?;
            new_value
        }

        ir::Value::Variable(name) => builder
            .get_var(&name)
            .and_then(|v| v.ok_or(TranslationError::UndeclaredVariable.into()))?,

        ir::Value::Scope(expr) => {
            builder.enter_new_scope();
            let content = translate_eir(builder, *expr)?;
            builder.exit_scope()?;
            content.into()
        }

        ir::Value::IfElse(cond, then_expr, else_expr) => {
            let condition_value = translate_eir(builder, *cond)?.expect_value()?;

            let then_block = builder.create_block()?;
            let else_block = builder.create_block()?;
            let merge_block = builder.create_block()?;

            let initial_block = builder.current_block()?;

            builder.switch_to_block(&then_block);
            let then_return = translate_eir(builder, *then_expr)?;

            builder.switch_to_block(&initial_block);
            let var_name = builder.declare_mut_var("__cond", &then_return, true)?;
            builder
                .brz(condition_value, &then_block, &else_block)?;

            builder.switch_to_block(&then_block);
            builder.assign_var(&var_name, &then_return)?;
            builder.jump(&merge_block);

            // Start writing 'else' block
            builder.switch_to_block(&else_block);
            let else_return = translate_eir(builder, *else_expr)?;
            builder.assign_var(&var_name, &else_return)?;

            // Jump to merge block after translation of the 'then' block
            builder.jump(&merge_block);

            builder.switch_to_block(&merge_block);
            builder.get_var(&var_name)?.unwrap().into()
        }
        _ => bail!(TranslationError::NotTyped),
    })
}
