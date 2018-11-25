//
// this code is based on https://github.com/nwoeanhinnogaehr/algorithmw-rust
//
// Copyright 2016 Noah Weninger
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use error::InternalError;
use expression::Operator;
use ir;
use transform::error::TypeInferError;
use transform::Transform;

use transform::type_infer::poly_type::PolyType;
use transform::type_infer::subst::Subst;
use transform::type_infer::traits::Types;
use transform::type_infer::type_::{Type, TypeVarGen};
use transform::type_infer::type_env::TypeEnv;

use failure::Error;

pub struct TypeInfer {
    tvg: TypeVarGen,
    instantiation_table: Vec<(Type, Type)>
}

impl TypeInfer {
    pub fn new() -> Self {
        Self {
            tvg: TypeVarGen::new(),
            instantiation_table: Vec::new()
        }
    }

    fn transform_with_env(
        &mut self,
        eir: &ir::Value,
        env: &mut TypeEnv,
    ) -> Result<(Subst, ir::Value), Error> {
        match eir {
            ir::Value::Typed(..) => Ok((Subst::new(), eir.clone())),
            ir::Value::Constant(_) => Err(TypeInferError::NotTyped.into()),
            ir::Value::Variable(ident) => match env.get(ident) {
                Some(s) => {
                    let instance = s.instantiate(&mut self.tvg);
                    self.instantiation_table.push((s.ty.clone(), instance.clone()));
                    Ok((Subst::new(), eir.with_type(instance)?))
                }
                None => Err(TypeInferError::UndeclaredIdentifier {
                    ident: ident.clone(),
                }.into()),
            },
            ir::Value::Function(ident, box body) => {
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
                let new_node = ir::Value::Function(ident.to_string(), box v.clone());
                Ok((s1.clone(), new_node.with_type(new_type)?))
            }
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
                    new_node.with_type(tv.apply(&s3))?,
                ))
            }
            ir::Value::Bind(kind, ident, box value) => {
                let (s1, v1) = self.transform_with_env(value, env)?;
                let t1 = v1.type_().unwrap();

                let tp = env.apply(&s1).generalize(&t1);
                env.insert(ident.clone(), tp);

                let new_node = ir::Value::Bind(kind.clone(), ident.clone(), box v1.clone());
                Ok((s1, new_node.with_type(t1.clone())?))
            }
            ir::Value::Scope(box body) => {
                let mut new_env = env.clone();
                let (s1, v1) = self.transform_with_env(body, &mut new_env)?;
                let t1 = v1.type_().unwrap();

                let new_node = ir::Value::Scope(box v1.clone());
                Ok((s1, new_node.with_type(t1.clone())?))
            }
            ir::Value::Follow(box lhs, box rhs) => {
                let (s1, v1) = self.transform_with_env(lhs, env)?;
                let (s2, v2) = self.transform_with_env(rhs, env)?;
                let t = v2.type_().unwrap();

                let new_node = ir::Value::Follow(box v1.clone(), box v2.clone());
                Ok((s1.compose(&s2), new_node.with_type(t.clone())?))
            }
            ir::Value::BinOp(op, box rhs, box lhs) => {
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
                            new_node.with_type(Type::Boolean)?,
                        )
                    }
                    _ => {
                        let sl = lhs_ty.mgu(&Type::Number)?;
                        let sr = rhs_ty.mgu(&Type::Number)?;
                        (
                            s1.compose(&s2.compose(&sl.compose(&sr))),
                            new_node.with_type(Type::Number)?,
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
                    new_node.with_type(then_ty.clone())?,
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
                    new_node.with_type(lhs_ty.clone())?,
                ))
            }
        }
    }
}

struct ApplySubst {
    subst: Subst,
    instantiation_table: Vec<(Type, Type)>,
}

impl ApplySubst {
    fn apply_all(&self, eir: &ir::Value) -> Result<Box<ir::Value>, Error> {
        match eir {
            ir::Value::Typed(ty, _, box value) => {
                let new_ty = ty.apply(&self.subst);
                let new_v = match value {
                    ir::Value::Constant(..) | ir::Value::Variable(..) => value.clone(),
                    ir::Value::Bind(kind, ident, box body) => {
                        ir::Value::Bind(*kind, ident.clone(), self.apply_all(body)?)
                    }
                    ir::Value::Assign(box lhs, box rhs) => {
                        ir::Value::Assign(self.apply_all(lhs)?, self.apply_all(rhs)?)
                    }
                    ir::Value::Scope(box body) => ir::Value::Scope(self.apply_all(body)?),
                    ir::Value::Follow(box lhs, box rhs) => {
                        ir::Value::Follow(self.apply_all(lhs)?, self.apply_all(rhs)?)
                    }
                    ir::Value::Apply(box lhs, box rhs) => {
                        ir::Value::Apply(self.apply_all(lhs)?, self.apply_all(rhs)?)
                    }
                    ir::Value::BinOp(op, box lhs, box rhs) => {
                        ir::Value::BinOp(*op, self.apply_all(lhs)?, self.apply_all(rhs)?)
                    }
                    ir::Value::IfElse(box cond, box then_v, box else_v) => ir::Value::IfElse(
                        self.apply_all(cond)?,
                        self.apply_all(then_v)?,
                        self.apply_all(else_v)?,
                    ),
                    ir::Value::Function(ident, box body) => {
                        ir::Value::Function(ident.clone(), self.apply_all(body)?)
                    }
                    ir::Value::Typed(..) => return Err(InternalError::DoubleTyped.into()),
                };
                Ok(box new_v.with_type(new_ty)?)
            }
            _ => Err(TypeInferError::NotTyped.into()),
        }
    }
}

impl Transform for TypeInfer {
    fn transform(&mut self, eir: &ir::Value) -> Result<ir::Value, Error> {
        let (subst, v) = self.transform_with_env(eir, &mut TypeEnv::new())?;
        let a = ApplySubst { subst: subst.clone() };
        let box v = a.apply_all(&v)?;
        println!("Candidates:");
        for (k, v) in self.instantiation_table.clone() {
            println!("susb: {} => {}", k.apply(&subst), v.apply(&subst));
        }
        Ok(v)
    }
}
