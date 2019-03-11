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

use super::traits::Types;

use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

/// A substitution is a mapping from type variables to types.
#[derive(Clone, Debug, Default)]
pub struct Subst(HashMap<TypeVarID, Type>);

impl Deref for Subst {
    type Target = HashMap<TypeVarID, Type>;
    fn deref(&self) -> &HashMap<TypeVarID, Type> {
        &self.0
    }
}
impl DerefMut for Subst {
    fn deref_mut(&mut self) -> &mut HashMap<TypeVarID, Type> {
        &mut self.0
    }
}

impl Subst {
    /// Construct an empty substitution.
    pub fn new() -> Subst {
        Subst(HashMap::new())
    }

    pub fn with_map(map: HashMap<TypeVarID, Type>) -> Subst {
        Subst(map)
    }

    /// To compose two substitutions, we apply self to each type in other and union the resulting
    /// substitution with self.
    pub fn compose(&self, other: &Subst) -> Subst {
        Subst(self.union(&other.iter().map(|(k, v)| (*k, v.apply(self))).collect()))
    }

    fn union(&self, other: &HashMap<TypeVarID, Type>) -> HashMap<TypeVarID, Type> {
        let mut res = self.0.clone();
        for (key, value) in other {
            res.entry(key.clone()).or_insert_with(|| value.clone());
        }
        res
    }

    fn remove_indirection_ty(&self, keys: &HashSet<TypeVarID>, ty: &Type) -> Type {
        if ty.ftv().is_disjoint(keys) {
            return ty.clone();
        }
        self.remove_indirection_ty(keys, &ty.apply(self))
    }

    pub fn remove_indirection(&self) -> Subst {
        let keys = self.keys().cloned().collect();
        Subst(
            self.iter()
                .map(|(k, v)| (k.clone(), self.remove_indirection_ty(&keys, v)))
                .collect(),
        )
    }
}
