use error::InternalError;
use expression::Operator;
use ir;
use scope::{Scope, ScopedEnv};
use transform::error::TypeInferError;
use transform::Transform;
use type_::type_::TypeData;
use type_::{PrimitiveKind, TypeID, TypeStore};

use failure::Error;

pub struct TypeInfer<'a> {
    type_store: &'a mut TypeStore,
    env: ScopedEnv<TypeID>,
}

impl<'a> TypeInfer<'a> {
    pub fn new(type_store: &'a mut TypeStore) -> Self {
        Self {
            type_store,
            env: ScopedEnv::new(),
        }
    }

    fn bin_op(&self, op: Operator, lhs: &ir::Value, rhs: &ir::Value) -> Result<ir::Value, Error> {
        let new_inst = ir::Value::BinOp(op, box lhs.clone(), box rhs.clone());
        if lhs.type_().is_none() || rhs.type_().is_none() {
            return Ok(new_inst);
        }

        let number_type = self.type_store.primitive(PrimitiveKind::Number);
        let boolean_type = self.type_store.primitive(PrimitiveKind::Boolean);
        Ok(match op {
            Operator::Index => unimplemented!(),
            Operator::Lt
            | Operator::Gt
            | Operator::Le
            | Operator::Ge
            | Operator::Eq
            | Operator::Ne => {
                self.check_type(lhs.type_().unwrap(), number_type)?;
                self.check_type(rhs.type_().unwrap(), number_type)?;
                ir::Value::Typed(boolean_type, box new_inst)
            }
            _ => {
                self.check_type(lhs.type_().unwrap(), number_type)?;
                self.check_type(rhs.type_().unwrap(), number_type)?;
                ir::Value::Typed(number_type, box new_inst)
            }
        })
    }

    fn type_data(&self, t: TypeID) -> Result<&TypeData, Error> {
        self.type_store
            .get(t)
            .ok_or(InternalError::InvalidTypeID.into())
    }

    fn type_data_mut(&mut self, t: TypeID) -> Result<&mut TypeData, Error> {
        self.type_store
            .get_mut(t)
            .ok_or(InternalError::InvalidTypeID.into())
    }

    fn prune(&self, t: TypeID) -> Result<TypeID, Error> {
        Ok(match self.type_data(t)? {
            TypeData::Variable(Some(v)) => v.clone(),
            _ => t,
        })
    }

    fn unify(&mut self, t1: TypeID, t2: TypeID) -> Result<(), Error> {
        let t1 = self.prune(t1)?;
        let t2 = self.prune(t2)?;

        if t1 == t2 {
            return Ok(());
        }

        match (self.type_data(t1)?.clone(), self.type_data(t2)?.clone()) {
            (TypeData::Variable(..), _) => {
                if let TypeData::Variable(ref mut instance) = self.type_data_mut(t1)? {
                    *instance = Some(t2);
                }
            }
            (_, TypeData::Variable(..)) => {
                self.unify(t2, t1)?;
            }
            (TypeData::Function(from1, to1), TypeData::Function(from2, to2)) => {
                self.unify(from1, from2)?;
                self.unify(to1, to2)?;
            }
            (_, _) => unimplemented!(),
        }
        Ok(())
    }

    fn type_of(val: &ir::Value) -> Result<TypeID, Error> {
        val.type_().ok_or(TypeInferError::NotTyped.into())
    }

    fn check_type(&self, expected: TypeID, t: TypeID) -> Result<TypeID, Error> {
        ensure!(
            expected == t,
            TypeInferError::MismatchedTypes { expected, found: t }
        );
        Ok(t)
    }
}

impl<'a> Transform for TypeInfer<'a> {
    fn transform(&mut self, eir: &ir::Value) -> Result<ir::Value, Error> {
        Ok(match eir {
            ir::Value::Typed(_, _) => eir.clone(),
            ir::Value::BinOp(op, box lhs, box rhs) => {
                let lhs = self.transform(&lhs)?;
                let rhs = self.transform(&rhs)?;
                self.bin_op(*op, &lhs, &rhs)?
            }
            ir::Value::Bind(kind, ident, box rhs) => {
                let rhs = self.transform(&rhs)?;
                let rhs_ty = Self::type_of(&rhs)?;
                self.env.insert(ident, rhs_ty);

                let new_inst = ir::Value::Bind(kind.clone(), ident.clone(), box rhs);

                ir::Value::Typed(rhs_ty, box new_inst)
            }
            ir::Value::Assign(box lhs, box rhs) => {
                let lhs = self.transform(&lhs)?;
                let rhs = self.transform(&rhs)?;

                let lhs_ty = Self::type_of(&lhs)?;
                let rhs_ty = Self::type_of(&rhs)?;

                self.check_type(lhs_ty, rhs_ty)?;

                let new_inst = ir::Value::Assign(box lhs, box rhs);

                ir::Value::Typed(rhs_ty, box new_inst)
            }
            ir::Value::Follow(box lhs, box rhs) => {
                let lhs = self.transform(&lhs)?;
                let rhs = self.transform(&rhs)?;
                let rhs_ty = Self::type_of(&rhs)?;

                let new_inst = ir::Value::Follow(box lhs, box rhs);

                ir::Value::Typed(rhs_ty, box new_inst)
            }
            ir::Value::Scope(box inside) => {
                let new_scope = self.env.new_scope();
                self.env.push(new_scope);
                let inside = self.transform(&inside)?;
                self.env.pop()?;

                let inside_ty = Self::type_of(&inside)?;

                let new_inst = ir::Value::Scope(box inside);

                ir::Value::Typed(inside_ty, box new_inst)
            }
            ir::Value::Variable(ident) => {
                let type_ = self
                    .env
                    .get(ident)
                    .ok_or(TypeInferError::UndeclaredIdentifier {
                        ident: ident.clone(),
                    })?;

                ir::Value::Typed(type_, box eir.clone())
            }
            ir::Value::IfElse(box cond, box then_, box else_) => {
                let cond = self.transform(&cond)?;
                let then_ = self.transform(&then_)?;
                let else_ = self.transform(&else_)?;

                let cond_ty = Self::type_of(&cond)?;
                let then_ty = Self::type_of(&then_)?;
                let else_ty = Self::type_of(&else_)?;

                let boolean_type = self.type_store.primitive(PrimitiveKind::Boolean);

                self.check_type(cond_ty, boolean_type)?;
                self.check_type(then_ty, else_ty)?;

                let new_inst = ir::Value::IfElse(box cond, box then_, box else_);

                ir::Value::Typed(then_ty, box new_inst)
            }
            ir::Value::Function(ident, box body) => {
                let param_ty = self.type_store.new_variable();
                let new_scope = self.env.new_scope();
                self.env.push(new_scope);
                self.env.insert(&ident, param_ty);
                let body = self.transform(&body)?;
                self.env.pop()?;
                let return_ty = Self::type_of(&body)?;

                let f_ty = self.type_store.new_function(param_ty, return_ty);
                ir::Value::Typed(f_ty, box eir.clone())
            }
            ir::Value::Apply(box lhs, box rhs) => {
                let lhs = self.transform(&lhs)?;
                let rhs = self.transform(&rhs)?;

                let lhs_ty = Self::type_of(&lhs)?;
                let rhs_ty = Self::type_of(&rhs)?;

                let result_ty = self.type_store.new_variable();
                let fn_ty = self.type_store.new_function(rhs_ty, result_ty);
                self.unify(fn_ty, lhs_ty)?;

                let new_inst = ir::Value::Apply(box lhs, box rhs);
                ir::Value::Typed(result_ty, box new_inst)
            }
            ir::Value::Constant(_) => bail!(TypeInferError::NotTyped),
        })
    }
}
