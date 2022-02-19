// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::{Compiler, Options};
use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};
use utilities::result::Result;

/// A WebAssembly module with compiled code.
#[derive(Debug, Serialize, Deserialize, Archive)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Module {
    options: Options,
}

impl Module {
    /// Creates a new `Module` with the given options.
    pub fn new(wasm: &[u8], options: Options) -> Result<Self> {
        // Create compiler and compile wasm bytes.
        let compiler = Compiler::new(options.liftoff);

        // Compile wasm bytes.
        compiler.compile(wasm)?;

        Ok(Self { options })
    }
}
