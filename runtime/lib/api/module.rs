// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::{compiler::Compiler, Imports, Instance, Options};
use serde::{Deserialize, Serialize};
use utilities::result::Result;

/// A WebAssembly module with compiled code but with unresolved external references.
/// Memories and tables are also not created yet.
///
/// Module is serializable and can be shared across threads.
#[derive(Debug, Serialize, Deserialize, Default)]
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

    /// Creates a WebAssembly instance.
    ///
    /// Resolves external references (globals, functions, memories, tables) and creates internal memories and tables.
    ///
    /// Four main operations performed are:
    /// 1. Creation of local memories and tables.
    /// 2. Initialization of local memories, tables and globals.
    /// 3. Resolving imported references.
    /// 4. Calling the start function.
    ///
    /// Imported function references are resolved using the method described [here](https://github.com/gigamono/wasmo/blob/main/IDEAS.md)
    pub fn initialize(&self, _imports: &Imports) -> Result<Instance> {
        // call_add_imported_function_resolver(imports) // function resolution done at runtime
        //
        // call_resolve_imported_memories(imports)
        // call_resolve_imported_tables(imports)
        // call_resolve_imported_globals(imports)
        //
        // call_initialize_local_memories() // data segment initializer
        // call_initialize_local_tables() // table segment initializer
        // call_initialize_local_globals() // global variable initializer
        //
        // call_start()

        // Each resolution updating the Store.

        todo!()
    }
}
