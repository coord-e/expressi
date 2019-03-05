use std::fmt;

#[derive(PartialEq, Debug, Clone, Eq, Copy)]
pub enum BindingKind {
    Mutable,
    Immutable,
}

impl fmt::Display for BindingKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BindingKind::Mutable => "mut",
                BindingKind::Immutable => "",
            }
        )
    }
}
