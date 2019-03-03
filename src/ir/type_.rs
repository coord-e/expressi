use crate::transform::type_infer::subst::Subst;
use crate::transform::type_infer::traits::Types;

use crate::transform::error::TypeInferError;

use failure::Error;

use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TypeVarID(usize);

impl TypeVarID {
    pub(crate) fn with_usize(id: usize) -> Self {
        TypeVarID(id)
    }

    /// Attempt to bind a type variable to a type, returning an appropriate substitution.
    pub fn bind(self, ty: &Type) -> Result<Subst, Error> {
        // Check for binding a variable to itself
        if let Type::Variable(ref u) = *ty {
            if u == &self {
                return Ok(Subst::new());
            }
        }

        // The occurs check prevents illegal recursive types.
        if ty.ftv().contains(&self) {
            return Err(TypeInferError::RecursiveType {
                t1: self,
                t2: ty.clone(),
            }
            .into());
        }

        let mut s = Subst::new();
        s.insert(self, ty.clone());
        Ok(s)
    }
}

impl fmt::Display for TypeVarID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Variable(TypeVarID),
    Number,
    Boolean,
    Empty,
    Function(Box<Type>, Box<Type>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Variable(id) => write!(f, "{}", id),
            Type::Function(box t1, box t2) => write!(f, "({} -> {})", t1, t2),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl Type {
    /// Most general unifier, a substitution S such that S(self) is congruent to S(other).
    pub fn mgu(&self, other: &Type) -> Result<Subst, Error> {
        match (self, other) {
            // For functions, we find the most general unifier for the inputs, apply the resulting
            // substitution to the outputs, find the outputs' most general unifier, and finally
            // compose the two resulting substitutions.
            (Type::Function(box in1, box out1), Type::Function(box in2, box out2)) => {
                let sub1 = in1.mgu(&in2)?;
                let sub2 = out1.apply(&sub1).mgu(&out2.apply(&sub1))?;
                Ok(sub1.compose(&sub2))
            }

            // If one of the types is variable, we can bind the variable to the type.
            // This also handles the case where they are both variables.
            (&Type::Variable(ref v), t) => v.bind(t),
            (t, &Type::Variable(ref v)) => v.bind(t),

            // If they are both primitives, no substitution needs to be done.
            (&Type::Number, &Type::Number)
            | (&Type::Boolean, &Type::Boolean)
            | (&Type::Empty, &Type::Empty) => Ok(Subst::new()),

            // Otherwise, the types cannot be unified.
            (t1, t2) => Err(TypeInferError::MismatchedTypes {
                expected: t2.clone(),
                found: t1.clone(),
            }
            .into()),
        }
    }
}
