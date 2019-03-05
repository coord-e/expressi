use super::{Atom, Builder};
use crate::error::{InternalError, TranslationError};
use crate::ir;

use failure::{bail, Error};
use inkwell::values::BasicValueEnum;
use std::collections::HashMap;

fn translate_monotype_function<'a>(
    builder: &mut Builder<'a>,
    param: String,
    ty: &ir::Type,
    body: ir::Node,
    capture_list: &HashMap<ir::Identifier, ir::Type>,
) -> Result<BasicValueEnum, Error> {
    let function = builder.function_constant(&ty, param, capture_list, |builder| {
        translate_eir(builder, body)?.expect_value()
    })?;
    Ok(function)
}

pub fn translate_eir<'a>(
    builder: &mut Builder<'a>,
    eir: ir::Node,
) -> Result<Atom<BasicValueEnum>, Error> {
    let ir::Node {
        value,
        type_,
        instantiation_table,
    } = eir;
    let ty = type_.ok_or(TranslationError::NotTyped)?;

    Ok(match value {
        ir::Value::Literal(c) => match c {
            ir::Literal::Number(number) => builder.number_constant(number)?.into(),
            ir::Literal::Boolean(tf) => builder.boolean_constant(tf)?.into(),
            ir::Literal::Empty => builder.empty_constant()?.into(),
            ir::Literal::Function(param, box body, capture_list) => {
                // TODO: Add more sufficient implementation to check whether PolyValue is needed or not
                if instantiation_table.len() <= 1 {
                    translate_monotype_function(builder, param, &ty, body, &capture_list)?.into()
                } else {
                    instantiation_table
                        .into_iter()
                        .map(|(ty, body)| {
                            match body {
                                ir::Value::Literal(ir::Literal::Function(_, box body, _)) => {
                                    translate_monotype_function(
                                        builder,
                                        param.clone(),
                                        &ty,
                                        body.clone(),
                                        &capture_list,
                                    )
                                }
                                _ => unreachable!(),
                            }
                            .map(|v| (ty.clone(), v))
                        })
                        .collect::<Result<HashMap<_, _>, _>>()?
                        .into()
                }
            }
        },
        ir::Value::Apply(box func, box arg) => {
            let func_ty = func.type_().ok_or(TranslationError::NotTyped)?;
            let func = translate_eir(builder, func.clone())?;
            let arg = translate_eir(builder, arg)?.expect_value()?;
            match func {
                Atom::LLVMValue(func) => builder.call(func, arg)?.into(),
                Atom::PolyValue(func_table) => builder.call(func_table[func_ty], arg)?.into(),
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

        ir::Value::Let(kind, name, box value, box body) => {
            let new_value = translate_eir(builder, value)?;

            builder.enter_new_scope();
            builder.bind_var(&name, &new_value, kind)?;
            let content = translate_eir(builder, body)?;
            builder.exit_scope()?;

            content
        }

        ir::Value::Assign(lhs, rhs) => {
            let new_value = translate_eir(builder, *rhs)?;
            let name = match lhs.value {
                ir::Value::Variable(name) => name,
                _ => panic!("Non-variable identifier"),
            };
            builder.assign_var(&name, &new_value)?;
            new_value
        }

        ir::Value::Variable(name) => builder
            .get_var(&name)
            .and_then(|v| v.ok_or_else(|| TranslationError::UndeclaredVariable.into()))?,

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
            builder.brz(condition_value, &then_block, &else_block)?;

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
            builder.get_var(&var_name)?.unwrap()
        }
    })
}
