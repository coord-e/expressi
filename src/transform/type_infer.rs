use expression::Operator;
use ir;
use transform::error::TypeInferError;
use transform::Transform;
use value::manager::PrimitiveKind;
use value::{TypeID, ValueManager};

use failure::Error;

use std::collections::HashMap;

struct Env(HashMap<String, TypeID>);

impl Env {
    fn new() -> Self {
        Env(HashMap::new())
    }
}

struct ScopedEnv(Vec<Env>);

impl ScopedEnv {
    fn new() -> Self {
        ScopedEnv(vec![Env::new()])
    }

    fn new_scope(&mut self) {
        self.0.push(Env::new());
    }

    fn exit_scope(&mut self) {
        self.0.pop();
    }

    fn merged(&self) -> HashMap<&String, &TypeID> {
        self.0.iter().flat_map(|env| env.0.iter()).collect()
    }

    fn insert(&mut self, key: &str, t: TypeID) {
        self.0.last_mut().unwrap().0.insert(key.to_string(), t);
    }

    fn get(&self, key: &String) -> Option<TypeID> {
        self.merged().get(key).cloned().cloned()
    }
}

pub struct TypeInfer {
    manager: ValueManager,
    env: ScopedEnv,
}

impl TypeInfer {
    pub fn new() -> Self {
        Self {
            manager: ValueManager::new(),
            env: ScopedEnv::new(),
        }
    }

    fn bin_op(&self, op: Operator, lhs: &ir::Value, rhs: &ir::Value) -> Result<ir::Value, Error> {
        let new_inst = ir::Value::BinOp(op, box lhs.clone(), box rhs.clone());
        if lhs.type_().is_none() || rhs.type_().is_none() {
            return Ok(new_inst);
        }

        let number_type = self.manager.primitive_type(PrimitiveKind::Number);
        let boolean_type = self.manager.primitive_type(PrimitiveKind::Boolean);
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

impl Transform for TypeInfer {
    fn transform(&mut self, eir: &ir::Value) -> Result<ir::Value, Error> {
        Ok(match eir {
            v @ ir::Value::Typed(_, _) => v.clone(),
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
                self.env.new_scope();
                let inside = self.transform(&inside)?;
                self.env.exit_scope();

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

                let boolean_type = self.manager.primitive_type(PrimitiveKind::Boolean);

                self.check_type(cond_ty, boolean_type)?;
                self.check_type(then_ty, else_ty)?;

                let new_inst = ir::Value::IfElse(box cond, box then_, box else_);

                ir::Value::Typed(then_ty, box new_inst)
            }
            _ => unimplemented!(),
        })
    }
}
