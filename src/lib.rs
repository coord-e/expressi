#![feature(box_into_raw_non_null)]

pub mod builder;
pub mod expression;
pub mod jit;
pub mod parser;
pub mod scope;
pub mod translator;
pub mod value;
pub mod ir;

pub mod error;

extern crate inkwell;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate serde_derive;

extern crate serde_yaml;

extern crate scopeguard;
