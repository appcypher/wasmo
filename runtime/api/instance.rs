use super::Store;
use crate::compiler::Value;
use crate::{Imports, Module};
use anyhow::Result;

/// An Instance is a fully resolved wasm runtime context.
/// External references (globals, functions, memories, tables) are resolved.
/// And memories and tables have been created.
#[derive(Debug, Default)]
pub struct Instance<'a> {
    _module: Option<&'a Module>,
    _store: Option<Store>,
}

impl<'a> Instance<'a> {
    /// Creates a WebAssembly instance.
    pub fn new(module: &'a Module, imports: &Imports) -> Result<Self> {
        module.initialize(imports, Default::default())
    }

    /// Invokes the function with the given name.
    pub fn invoke(_name: String, _params: &[Value]) -> Result<Value> {
        // TODO(appcypher): Implement this.
        todo!()
    }
}
