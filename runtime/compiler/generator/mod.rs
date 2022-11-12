mod function;
mod operator;

use anyhow::Result;
pub(crate) use function::*;
pub(crate) use operator::*;

/// Generates LLVM code for a construct.
pub(crate) trait Generator {
    type Value;

    fn generate(&mut self) -> Result<Self::Value>;
}
