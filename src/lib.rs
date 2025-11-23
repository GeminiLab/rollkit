mod ast;
mod eval;
mod parser;

pub mod parsing {
    pub use crate::{ast::*, parser::*};
}

pub use eval::{EvalError, Value, eval};
pub use parser::parse;
