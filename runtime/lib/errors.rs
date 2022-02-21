// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerError {
    UnsupportedInstanceType,
    UnsupportedModuleType,
    UnsupportedValType(wasmparser::Type),
}

impl std::error::Error for CompilerError {}

impl Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
