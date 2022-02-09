// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::{Exports, Module, Imports};

#[derive(Debug)]
pub struct Instance {}

impl Instance {
    pub fn new(_module: &Module, _imports: Imports) -> Self {
        // TODO: Compile the module.
        Self {}
    }

    fn _exports(_wasm_bytes: &[u8]) -> Exports {
        // TODO(appcypher): Validate the wasm bytes.
        unimplemented!()
    }
}
