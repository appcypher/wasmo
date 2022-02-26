// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::module::Module;
use utilities::result::Result;

/// An Instance is a fully resolved wasm runtime context.
/// External references (globals, functions, memories, tables) are resolved.
/// And memories and tables have been created.
#[derive(Debug)]
pub struct Instance<'a> {
    _module: &'a Module,
}

impl<'a> Instance<'a> {
    /// Creates a new instance.
    /// Resolves external references (globals, functions, memories, tables) and creates memories and tables.
    ///
    /// Three main operations are performed:
    /// 1. Creation of memories and tables.
    /// 2. Initialization of memories and tables from active data and elements.
    /// 3. Linking; resolving external references using the details mentioned [here](https://github.com/gigamono/wasmo/blob/main/IDEAS.md)
    pub fn new(module: &'a Module) -> Result<Self> {
        Ok(Self { _module: module })
    }
}
