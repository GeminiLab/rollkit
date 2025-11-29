mod ast;
mod eval;
mod parser;

/// Module containing all parsing-related functionality.
pub mod parsing {
    pub use crate::{ast::*, eval::range_to_iter, parser::*};
}

pub use eval::{EvalError, Value, eval, eval_with};
pub use parser::parse;
