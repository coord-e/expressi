//
// this code is based on https://github.com/nwoeanhinnogaehr/algorithmw-rust
//
// Copyright 2016 Noah Weninger
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use super::poly_type::PolyType;
use super::subst::Subst;
use super::traits::Types;
use super::type_::{Type, TypeVarID};

use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub struct TypeEnv(HashMap<String, PolyType>);

impl Deref for TypeEnv {
    type Target = HashMap<String, PolyType>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TypeEnv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Types for TypeEnv {
    /// The free type variables of a type environment is the union of the free type variables of
    /// each polytype in the environment.
    fn ftv(&self) -> HashSet<TypeVarID> {
        self.values().cloned().collect::<Vec<PolyType>>().ftv()
    }

    /// To apply a substitution, we just apply it to each polytype in the type environment.
    fn apply(&self, s: &Subst) -> TypeEnv {
        TypeEnv(self.iter().map(|(k, v)| (k.clone(), v.apply(s))).collect())
    }
}

impl TypeEnv {
    /// Construct an empty type environment.
    pub fn new() -> TypeEnv {
        TypeEnv(HashMap::new())
    }

    /// Generalize creates a polytype
    pub fn generalize(&self, ty: &Type) -> PolyType {
        PolyType {
            vars: ty.ftv().difference(&self.ftv()).cloned().collect(),
            ty: ty.clone(),
        }
    }
}
