pub mod poly_type;
pub mod subst;
pub mod traits;
pub mod type_;
pub mod type_env;
pub mod type_infer;

pub use crate::transform::type_infer::type_::Type;
pub use crate::transform::type_infer::type_infer::TypeInfer;
