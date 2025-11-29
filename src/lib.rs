#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

mod ast;
mod eval;
mod parser;

/// Module containing all parsing-related functionality.
pub mod parsing {
    pub use crate::{ast::*, parser::*};
}

pub use eval::{EvalError, Value, eval_with};
pub use parser::parse;

#[cfg(feature = "std")]
pub use eval::eval;
