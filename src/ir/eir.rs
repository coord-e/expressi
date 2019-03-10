use crate::error::InternalError;
use crate::expression::Operator;
use crate::transform::Transform;

use failure::Error;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;

pub type Identifier = String;
