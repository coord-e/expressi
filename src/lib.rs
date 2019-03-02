#![feature(box_syntax, box_patterns)]

pub mod expression;
pub mod ir;
pub mod jit;
pub mod parser;
pub mod scope;
pub mod transform;
pub mod translator;

pub mod error;

extern crate inkwell;

#[macro_use]
extern crate failure;

extern crate scopeguard;
