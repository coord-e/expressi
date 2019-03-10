pub mod inference;
pub mod poly_type;
pub mod subst;
pub mod traits;
pub mod type_;
pub mod type_env;
pub mod type_var_gen;

pub use self::inference::TypeInfer;
