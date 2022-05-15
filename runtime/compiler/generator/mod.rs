mod function;
mod operator;

pub(crate) use function::*;
pub(crate) use operator::*;

use anyhow::Result;

/// Generates LLVM code for a construct.
pub(crate) trait Generator {
    fn generate(&mut self) -> Result<()>;
}
