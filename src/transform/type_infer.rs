use expression::Operator;
use ir;
use transform::error::TypeInferError;
use transform::Transform;
use value::manager::PrimitiveKind;
use value::{TypeID, ValueManager};

use failure::Error;

use std::collections::HashMap;

struct Env(HashMap<String, Option<TypeID>>);

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

    fn merged(&self) -> HashMap<&String, &Option<TypeID>> {
        self.0.iter().flat_map(|env| env.0.iter()).collect()
    }

    fn insert(&mut self, key: &str, t: Option<TypeID>) {
        self.0.last_mut().unwrap().0.insert(key.to_string(), t);
    }

    fn get(&self, key: &String) -> Option<Option<TypeID>> {
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
        Ok(match op {
            Operator::Index => unimplemented!(),
            op @ _ => {
                self.check_type(lhs.type_().unwrap(), number_type)?;
                self.check_type(rhs.type_().unwrap(), number_type)?;
                ir::Value::Typed(number_type, box new_inst)
            }
        })
    }

    fn with_type(&self, type_: Option<TypeID>, new_inst: &ir::Value) -> Result<ir::Value, Error> {
        Ok(type_
            .map(|t| ir::Value::Typed(t, box new_inst.clone()))
            .unwrap_or_else(|| new_inst.clone()))
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
        match eir {
            v @ ir::Value::Typed(_, _) => Ok(v.clone()),
            ir::Value::BinOp(op, box lhs, box rhs) => {
                let lhs = self.transform(&lhs)?;
                let rhs = self.transform(&rhs)?;
                self.bin_op(*op, &lhs, &rhs)
            }
            ir::Value::Bind(kind, ident, box rhs) => {
                let rhs = self.transform(&rhs)?;
                let rhs_ty = rhs.type_();
                self.env.insert(ident, rhs_ty);

                let new_inst = ir::Value::Bind(kind.clone(), ident.clone(), box rhs);

                self.with_type(rhs_ty, &new_inst)
            }
            ir::Value::Follow(box lhs, box rhs) => {
                let lhs = self.transform(&lhs)?;
                let rhs = self.transform(&rhs)?;
                let rhs_ty = rhs.type_();

                let new_inst = ir::Value::Follow(box lhs, box rhs);

                self.with_type(rhs_ty, &new_inst)
            }
            _ => unimplemented!()
        }
    }
}

