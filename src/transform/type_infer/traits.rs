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

use failure::Error;

use std::collections::HashSet;

pub trait Types {
    fn ftv(&self) -> HashSet<TypeVarID>;
    fn apply(&self, _: &Subst) -> Self;
}

impl<'a, T> Types for Vec<T>
where
    T: Types,
{
    // The free type variables of a vector of types is the union of the free type variables of each
    // of the types in the vector.
    fn ftv(&self) -> HashSet<TypeVarID> {
        self.iter()
            .map(Types::ftv)
            .fold(HashSet::new(), |set, x| set.union(&x).cloned().collect())
    }

    // To apply a substitution to a vector of types, just apply to each type in the vector.
    fn apply(&self, s: &Subst) -> Vec<T> {
        self.iter().map(|x| x.apply(s)).collect()
    }
}

pub trait Bind {
    fn bind(self, ty: &Type) -> Result<Subst, Error>;
}

pub trait Unify {
    fn mgu(&self, other: &Type) -> Result<Subst, Error>;
}
