// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::{compiler::Compiler, Options};
use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};
use utilities::result::Result;

/// A WebAssembly module with compiled code but with unresolved external references.
/// Memories and tables are also not created yet.
///
/// Module is serializable and can be shared across threads.
#[derive(Debug, Serialize, Deserialize, Archive)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Module {
    options: Options,
    compiler: Compiler,
}

impl Module {
    /// Creates a new `Module` with the given options.
    pub fn new(wasm: &[u8], options: Options) -> Result<Self> {
        // Create compiler and compile wasm bytes.
        let mut compiler = Compiler::new(options.liftoff);

        // Compile wasm bytes.
        compiler.compile(wasm)?;

        Ok(Self { options, compiler })
    }
}
