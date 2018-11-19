use ir;

use failure::Error;

pub trait Transform {
    fn transform(&mut self, eir: &ir::Value) -> Result<ir::Value, Error>;
}
