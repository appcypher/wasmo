// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::{store::Store, Imports, Module};
use utilities::result::Result;

/// An Instance is a fully resolved wasm runtime context.
/// External references (globals, functions, memories, tables) are resolved.
/// And memories and tables have been created.
#[derive(Debug)]
pub struct Instance<'a> {
    _module: &'a Module,
    _store: Store,
}

impl<'a> Instance<'a> {
    /// Creates a WebAssembly instance.
    pub fn new(module: &'a Module, imports: &Imports) -> Result<Self> {
        module.initialize(imports)
    }

    // pub fn invoke(_name: String, params: Vec<Value>) -> Result<Value> {
    //     todo!()
    // }
}
