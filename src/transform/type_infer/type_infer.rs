//
// this code is based on https://github.com/nwoeanhinnogaehr/algorithmw-rust
//
// Copyright 2016 Noah Weninger
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

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
}

impl TypeInfer {
    pub fn new() -> Self {
        Self {
            tvg: TypeVarGen::new(),
        }
    }

    fn transform_with_env(
        &mut self,
        eir: &ir::Value,
        env: &mut TypeEnv,
    ) -> Result<(Subst, ir::Value), Error> {
        match eir {
            ir::Value::Typed(_, _) => Ok((Subst::new(), eir.clone())),
            ir::Value::Constant(_) => Err(TypeInferError::NotTyped.into()),
            ir::Value::Variable(ident) => {
                match env.get(ident) {
                    Some(s) => Ok((Subst::new(), eir.with_type(s.instantiate(&mut self.tvg))?)),
                    None => Err(TypeInferError::UndeclaredIdentifier {
                        ident: ident.clone(),
                    }.into()),
                }
            }
            ir::Value::Function(ident, box body) => {
                let tv = self.tvg.new_variable();
                let mut new_env = env.clone();
                new_env.remove(ident);
                new_env.insert(ident.clone(),
                PolyType {
                    vars: Vec::new(),
                    ty: tv.clone(),
                });
                let (s1, v) = self.transform_with_env(body, &mut new_env)?;
                let t1 = v.type_().unwrap();
                let new_type = Type::Function(box tv.apply(&s1), box t1.clone());
                Ok((s1.clone(), eir.with_type(new_type)?))
            }
            ir::Value::Apply(box f, box arg) => {
                let (s1, v1) = self.transform_with_env(f, env)?;
                let t1 = v1.type_().unwrap();
                let (s2, v2) = self.transform_with_env(arg, &mut env.apply(&s1))?;
                let t2 = v2.type_().unwrap();

                let tv = self.tvg.new_variable();
                let s3 = t1.apply(&s2).mgu(&Type::Function(box t2.clone(), box tv.clone()))?;
                Ok((s3.compose(&s2.compose(&s1)), eir.with_type(tv.apply(&s3))?))
            }
            ir::Value::Bind(_, ident, box value) => {
                let (s1, v1) = self.transform_with_env(value, env)?;
                let t1 = v1.type_().unwrap();

                let tp = env.apply(&s1).generalize(&t1);
                env.insert(ident.clone(), tp);
                Ok((s1, eir.with_type(t1.clone())?))
            }
            ir::Value::Scope(box body) => {
                let mut new_env = env.clone();
                let (s1, v1) = self.transform_with_env(body, &mut new_env)?;
                let t1 = v1.type_().unwrap();

                Ok((s1, eir.with_type(t1.clone())?))
            }
            ir::Value::Follow(box lhs, box rhs) => {
                let (s1, _) = self.transform_with_env(lhs, env)?;
                let (s2, v) = self.transform_with_env(rhs, env)?;
                let t = v.type_().unwrap();

                Ok((s1.compose(&s2), eir.with_type(t.clone())?))
            }
            ir::Value::BinOp(op, box rhs, box lhs) => {
                let (s1, lhs) = self.transform_with_env(&lhs, env)?;
                let lhs_ty = lhs.type_().unwrap();
                let (s2, rhs) = self.transform_with_env(&rhs, env)?;
                let rhs_ty = rhs.type_().unwrap();
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
                        (s1.compose(&s2.compose(&sl.compose(&sr))), eir.with_type(Type::Boolean)?)
                    }
                    _ => {
                        let sl = lhs_ty.mgu(&Type::Number)?;
                        let sr = rhs_ty.mgu(&Type::Number)?;
                        (s1.compose(&s2.compose(&sl.compose(&sr))), eir.with_type(Type::Number)?)
                    }
                })
            }
            _ => unimplemented!()
            // Assign(Box<Value>, Box<Value>),
            // IfElse(Box<Value>, Box<Value>, Box<Value>),
        }
    }
}

impl Transform for TypeInfer {
    fn transform(&mut self, eir: &ir::Value) -> Result<ir::Value, Error> {
        let (_, v) = self.transform_with_env(eir, &mut TypeEnv::new())?;
        Ok(v)
    }
}
