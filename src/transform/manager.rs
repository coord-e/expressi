use super::{CheckCapture, Transform, TypeInfer};
use crate::ir;

use failure::Error;

use std::default::Default;

pub struct TransformManager {
    transforms: Vec<Box<dyn Transform>>,
}

impl TransformManager {
    pub fn apply(&mut self, eir: ir::Node) -> Result<ir::Node, Error> {
        self.transforms
            .iter_mut()
            .try_fold(eir, |ir, t| t.transform(&ir))
    }
}

impl Default for TransformManager {
    fn default() -> Self {
        TransformManager {
            transforms: vec![box TypeInfer::new(), box CheckCapture::new()],
        }
    }
}
