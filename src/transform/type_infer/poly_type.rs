//
// this code is based on https://github.com/nwoeanhinnogaehr/algorithmw-rust
//
// Copyright 2016 Noah Weninger
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use transform::type_infer::subst::Subst;
use transform::type_infer::traits::Types;
use transform::type_infer::type_::{Type, TypeVarGen, TypeVarID};

use std::collections::HashSet;

/// A polytype is a type in which there are a number of for-all quantifiers, i.e. some parts of the
/// type may not be concrete but instead correct for all possible types.
#[derive(Clone, Debug)]
pub struct PolyType {
    pub vars: Vec<TypeVarID>,
    pub ty: Type,
}

impl Types for PolyType {
    /// The free type variables in a polytype are those that are free in the internal type and not
    /// bound by the variable mapping.
    fn ftv(&self) -> HashSet<TypeVarID> {
        self.ty
            .ftv()
            .difference(&self.vars.iter().cloned().collect())
            .cloned()
            .collect()
    }

    /// Substitutions are applied to free type variables only.
    fn apply(&self, s: &Subst) -> PolyType {
        PolyType {
            vars: self.vars.clone(),
            ty: {
                let mut sub = s.clone();
                for var in &self.vars {
                    sub.remove(var);
                }
                self.ty.apply(&sub)
            },
        }
    }
}

impl PolyType {
    /// Instantiates a polytype into a type. Replaces all bound type variables with fresh type
    /// variables and return the resulting type.
    pub fn instantiate(&self, tvg: &mut TypeVarGen) -> Type {
        let newvars = self.vars.iter().map(|_| tvg.new_variable());
        self.ty.apply(&Subst::with_map(
            self.vars.iter().cloned().zip(newvars).collect(),
        ))
    }
}
