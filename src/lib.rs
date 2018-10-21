#![feature(box_into_raw_non_null)]

pub mod builder;
pub mod expression;
pub mod jit;
pub mod parser;
pub mod translator;
pub mod value;
pub mod type_;
pub mod scope;

pub mod error;

extern crate inkwell;

#[macro_use]
extern crate failure;

extern crate scopeguard;
