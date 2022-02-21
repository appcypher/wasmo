// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::module::Module;
use utilities::result::Result;

/// An Instance is a fully resolved wasm runtime context.
/// External references (globals, functions, memories, tables) are resolved.
/// And memories and tables have been created.
#[derive(Debug)]
pub struct Instance {}

impl Instance {
    /// Creates a new instance.
    /// Resolves external references (globals, functions, memories, tables) and creates memories and tables.
    ///
    /// Three main operations are performed:
    /// 1. Creation of memories and tables.
    /// 2. Initialization of memories and tables.
    /// 3. Linking which is the resolution of external references.
    pub fn new(_module: &Module) -> Result<Self> {
        Ok(Self {})
    }
}
