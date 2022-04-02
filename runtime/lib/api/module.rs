use crate::{compiler::Compiler, Imports, Instance, Options, Store};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// A WebAssembly module with compiled code but with unresolved external references.
/// Memories and tables are also not created yet.
///
/// Module is serializable and can be shared across threads.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Module {
    pub options: Options,
    compiler: Compiler,
}

/// Options available for initialiazing a module.
#[derive(Debug, Default)]
pub struct InitializeOpts {
    store: Option<Store>,
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
    /// Resolves and initialises the instance.
    ///
    /// Five main operations performed are:
    /// 1. Fix up resolver address.
    /// 2. Resolve imported memories, tables and globals.
    /// 3. Create local memories, tables and globals.
    /// 4. Populate memories, tables and globals.
    /// 5. Call start function.
    pub fn initialize(&self, _imports: &Imports, _opts: InitializeOpts) -> Result<Instance> {
        // TODO(appcypher): Create Store or use the one in opts.
        // TODO(appcypher): Implement.
        todo!()
    }
}
