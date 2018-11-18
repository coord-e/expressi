use ir;

use failure::Error;

pub trait Transform {
    fn transform(&self, eir: &ir::Value) -> Result<ir::Value, Error>;
}
