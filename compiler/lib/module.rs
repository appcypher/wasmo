// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.
use crate::CompileMode;
use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};
use utilities::result::Result;

/// A WebAssembly module with compiled code.
#[derive(Debug, Serialize, Deserialize, Archive)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Module {}

impl Module {
    pub fn new(_wasm_bytes: &[u8], _mode: CompileMode) -> Self {
        // TODO: Compile the module.
        Self {}
    }

    fn _validate_wasm(_wasm_bytes: &[u8]) -> Result<()> {
        // TODO(appcypher): Validate the wasm bytes.
        Ok(())
    }

    fn _compile(_wasm_bytes: &[u8]) -> () {
        // TODO(appcypher): Compile the wasm bytes.
        // Compiled LLVM Module ?
    }
}
