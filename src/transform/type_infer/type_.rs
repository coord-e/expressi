//
// this code is based on https://github.com/nwoeanhinnogaehr/algorithmw-rust
//
// Copyright 2016 Noah Weninger
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use crate::ir::type_::{Type, TypeVarID};

use super::subst::Subst;
use super::traits::Types;

use std::collections::HashSet;

pub struct TypeVarGen {
    supply: usize,
}

impl TypeVarGen {
    pub fn new() -> TypeVarGen {
        TypeVarGen { supply: 0 }
    }
    pub fn next(&mut self) -> TypeVarID {
        let v = TypeVarID::with_usize(self.supply);
        self.supply += 1;
        v
    }

    pub fn new_variable(&mut self) -> Type {
        Type::Variable(self.next())
    }
}

impl Types for Type {
    fn ftv(&self) -> HashSet<TypeVarID> {
        match self {
            // For a type variable, there is one free variable: the variable itself.
            &Type::Variable(ref s) => [*s].iter().cloned().collect(),

            // Primitive types have no free variables
            &Type::Number | &Type::Boolean | &Type::Empty => HashSet::new(),

            // For functions, we take the union of the free type variables of the input and output.
            Type::Function(box i, box o) => i.ftv().union(&o.ftv()).cloned().collect(),
        }
    }

    fn apply(&self, s: &Subst) -> Type {
        match self {
            // If this type references a variable that is in the substitution, return it's
            // replacement type. Otherwise, return the existing type.
            &Type::Variable(ref n) => s.get(n).cloned().unwrap_or_else(|| self.clone()),

            // To apply to a function, we simply apply to each of the input and output.
            Type::Function(box t1, box t2) => Type::Function(box t1.apply(s), box t2.apply(s)),

            // A primitive type is changed by a substitution.
            _ => self.clone(),
        }
    }
}
