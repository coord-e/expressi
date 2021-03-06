//
// this code is based on https://github.com/nwoeanhinnogaehr/algorithmw-rust
//
// Copyright 2016 Noah Weninger
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use crate::expression::Operator;
use crate::ir;
use crate::ir::type_::Type;
use crate::transform::error::TypeInferError;
use crate::transform::Transform;

use super::poly_type::PolyType;
use super::subst::Subst;
use super::traits::{Types, Unify};
use super::type_env::TypeEnv;
use super::type_var_gen::TypeVarGen;

use failure::Error;

#[derive(Default)]
pub struct TypeInfer {
    tvg: TypeVarGen,
    instantiation_table: Vec<(Type, Subst)>,
}

impl TypeInfer {
    pub fn new() -> Self {
        Self {
            tvg: TypeVarGen::new(),
            instantiation_table: Vec::new(),
        }
    }

    fn transform_with_env(
        &mut self,
        eir: &ir::Node,
        env: &mut TypeEnv,
    ) -> Result<(Subst, ir::Node), Error> {
        if eir.type_().is_some() {
            return Ok((Subst::new(), eir.clone()));
        }

        match eir.value() {
            ir::Value::Literal(c) => match c {
                ir::Literal::Function(ident, box body, captures) => {
                    let tv = self.tvg.new_variable();
                    let mut new_env = env.clone();
                    new_env.remove(ident);
                    new_env.insert(
                        ident.clone(),
                        PolyType {
                            vars: Vec::new(),
                            ty: tv.clone(),
                        },
                    );
                    let (s1, v) = self.transform_with_env(body, &mut new_env)?;
                    let t1 = v.type_().unwrap();
                    let new_type = Type::Function(box tv.apply(&s1), box t1.clone());
                    let lit =
                        ir::Literal::Function(ident.to_string(), box v.clone(), captures.clone());
                    let new_node = ir::Value::Literal(lit);
                    Ok((s1.clone(), new_node.typed_node(new_type)))
                }
                ir::Literal::Number(_) => Ok((Subst::new(), eir.clone().with_type(Type::Number)?)),
                ir::Literal::Boolean(_) => {
                    Ok((Subst::new(), eir.clone().with_type(Type::Boolean)?))
                }
                ir::Literal::Empty => Ok((Subst::new(), eir.clone().with_type(Type::Empty)?)),
            },
            ir::Value::Variable(ident) => match env.get(ident) {
                Some(s) => {
                    let (subst, instance) = s.instantiate(&mut self.tvg);
                    self.instantiation_table.push((s.ty.clone(), subst));
                    Ok((Subst::new(), eir.clone().with_type(instance)?))
                }
                None => Err(TypeInferError::UndeclaredIdentifier {
                    ident: ident.clone(),
                }
                .into()),
            },
            ir::Value::Apply(box f, box arg) => {
                let (s1, v1) = self.transform_with_env(f, env)?;
                let t1 = v1.type_().unwrap();
                let (s2, v2) = self.transform_with_env(arg, &mut env.apply(&s1))?;
                let t2 = v2.type_().unwrap();

                let tv = self.tvg.new_variable();
                let s3 = t1
                    .apply(&s2)
                    .mgu(&Type::Function(box t2.clone(), box tv.clone()))?;

                let new_node = ir::Value::Apply(box v1.clone(), box v2.clone());
                Ok((
                    s3.compose(&s2.compose(&s1)),
                    new_node.typed_node(tv.apply(&s3)),
                ))
            }
            ir::Value::Let(kind, ident, box value, box body) => {
                let (s1, v1) = self.transform_with_env(value, env)?;
                let t1 = v1.type_().unwrap();

                let tp = env.apply(&s1).generalize(&t1);
                env.insert(ident.clone(), tp);

                let (s2, v2) = self.transform_with_env(&body, &mut env.apply(&s1))?;
                let t2 = v2.type_().unwrap();

                let new_node = ir::Value::Let(*kind, ident.clone(), box v1.clone(), box v2.clone());
                Ok((s2.compose(&s1), new_node.typed_node(t2.clone())))
            }
            ir::Value::Follow(box lhs, box rhs) => {
                let (s1, v1) = self.transform_with_env(lhs, env)?;
                let (s2, v2) = self.transform_with_env(rhs, env)?;
                let t = v2.type_().unwrap();

                let new_node = ir::Value::Follow(box v1.clone(), box v2.clone());
                Ok((s1.compose(&s2), new_node.typed_node(t.clone())))
            }
            ir::Value::BinOp(op, box lhs, box rhs) => {
                let (s1, lhs) = self.transform_with_env(&lhs, env)?;
                let lhs_ty = lhs.type_().unwrap();
                let (s2, rhs) = self.transform_with_env(&rhs, env)?;
                let rhs_ty = rhs.type_().unwrap();

                let new_node = ir::Value::BinOp(*op, box lhs.clone(), box rhs.clone());
                Ok(match op {
                    Operator::Index => unimplemented!(),
                    Operator::Lt
                    | Operator::Gt
                    | Operator::Le
                    | Operator::Ge
                    | Operator::Eq
                    | Operator::Ne => {
                        let sl = lhs_ty.mgu(&Type::Number)?;
                        let sr = rhs_ty.mgu(&Type::Number)?;
                        (
                            s1.compose(&s2.compose(&sl.compose(&sr))),
                            new_node.typed_node(Type::Boolean),
                        )
                    }
                    _ => {
                        let sl = lhs_ty.mgu(&Type::Number)?;
                        let sr = rhs_ty.mgu(&Type::Number)?;
                        (
                            s1.compose(&s2.compose(&sl.compose(&sr))),
                            new_node.typed_node(Type::Number),
                        )
                    }
                })
            }
            ir::Value::IfElse(box cond, box then_body, box else_body) => {
                let (s1, cond_v) = self.transform_with_env(&cond, env)?;
                let cond_ty = cond_v.type_().unwrap();
                let (s2, then_v) = self.transform_with_env(&then_body, env)?;
                let then_ty = then_v.type_().unwrap();
                let (s3, else_v) = self.transform_with_env(&else_body, env)?;
                let else_ty = else_v.type_().unwrap();

                let cond_s = cond_ty.mgu(&Type::Boolean)?;
                let body_s = then_ty.mgu(&else_ty)?;

                let new_node =
                    ir::Value::IfElse(box cond_v.clone(), box then_v.clone(), box else_v.clone());
                Ok((
                    s1.compose(&s2.compose(&s3.compose(&cond_s.compose(&body_s)))),
                    new_node.typed_node(then_ty.clone()),
                ))
            }
            ir::Value::Assign(box lhs, box rhs) => {
                let (s1, lhs) = self.transform_with_env(&lhs, env)?;
                let lhs_ty = lhs.type_().unwrap();
                let (s2, rhs) = self.transform_with_env(&rhs, env)?;
                let rhs_ty = rhs.type_().unwrap();

                let subst = lhs_ty.mgu(&rhs_ty)?;

                let new_node = ir::Value::Assign(box lhs.clone(), box rhs.clone());
                Ok((
                    s1.compose(&s2.compose(&subst)),
                    new_node.typed_node(lhs_ty.clone()),
                ))
            }
        }
    }

    fn inner_apply_subst_all(&self, value: &ir::Value, subst: &Subst) -> Result<ir::Value, Error> {
        Ok(match value {
            ir::Value::Literal(lit) => match lit {
                ir::Literal::Function(ident, box body, captures) => {
                    ir::Value::Literal(ir::Literal::Function(
                        ident.clone(),
                        self.apply_subst_all(body, subst)?,
                        captures.clone(),
                    ))
                }
                _ => value.clone(),
            },
            ir::Value::Variable(..) => value.clone(),
            ir::Value::Let(kind, ident, box value, box body) => ir::Value::Let(
                *kind,
                ident.clone(),
                self.apply_subst_all(value, subst)?,
                self.apply_subst_all(body, subst)?,
            ),
            ir::Value::Assign(box lhs, box rhs) => ir::Value::Assign(
                self.apply_subst_all(lhs, subst)?,
                self.apply_subst_all(rhs, subst)?,
            ),
            ir::Value::Follow(box lhs, box rhs) => ir::Value::Follow(
                self.apply_subst_all(lhs, subst)?,
                self.apply_subst_all(rhs, subst)?,
            ),
            ir::Value::Apply(box lhs, box rhs) => ir::Value::Apply(
                self.apply_subst_all(lhs, subst)?,
                self.apply_subst_all(rhs, subst)?,
            ),
            ir::Value::BinOp(op, box lhs, box rhs) => ir::Value::BinOp(
                *op,
                self.apply_subst_all(lhs, subst)?,
                self.apply_subst_all(rhs, subst)?,
            ),
            ir::Value::IfElse(box cond, box then_v, box else_v) => ir::Value::IfElse(
                self.apply_subst_all(cond, subst)?,
                self.apply_subst_all(then_v, subst)?,
                self.apply_subst_all(else_v, subst)?,
            ),
        })
    }

    fn apply_subst_all(&self, eir: &ir::Node, subst: &Subst) -> Result<Box<ir::Node>, Error> {
        let ty = eir.type_().ok_or(TypeInferError::NotTyped)?;
        let value = eir.value();

        let new_ty = ty.apply(subst);
        let instantiation_table = self
            .instantiation_table
            .iter()
            .filter_map(|(k, v)| {
                if k == &new_ty {
                    let c = subst.compose(v).remove_indirection();
                    let instance_value = self.inner_apply_subst_all(value, &c).unwrap();
                    let applied_ty = ty.apply(&c);
                    Some((applied_ty.clone(), instance_value.typed_node(applied_ty)))
                } else {
                    None
                }
            })
            .collect();
        let new_v = self.inner_apply_subst_all(value, subst)?;
        Ok(box ir::Node::new(new_v, new_ty, instantiation_table))
    }
}

impl Transform for TypeInfer {
    fn transform(&mut self, eir: &ir::Node) -> Result<ir::Node, Error> {
        let (subst, v) = self.transform_with_env(eir, &mut TypeEnv::new())?;
        let box v = self.apply_subst_all(&v, &subst)?;
        Ok(v)
    }
}
