use std::pin::Pin;

use super::{context::LLContext, module::LLModule, types::LLFunctionType};
use anyhow::Result;
// use llvm_sys::core::LLVMShutdown;

/// Converts WebAssembly semantics to LLVM code and handles materialization.
///
/// # Safety
/// This type is self-referential so we can only construct it as a pinned object.
/// This prevents pointer issues that comes with moving the object.
///
/// The codegen phase generates some boilerplate code that is used during initialization, and execution.
/// They include:
///
/// #### The Resolvers Functions
/// - `resolve_functions(store_addr)` // if just-in-time function resolution is disabled. Resolves imported/builtin functions.
/// - `resolve_memories(store_addr)` // call builtins, save address to designated data section. Resolves imported and local memories.
/// - `resolve_tables(store_addr)` // ...
/// - `resolve_globals(store_addr)` // ...
///
/// #### The Initializer Functions
/// - `initialize(store_addr)` // call resolver functions.
/// - `initialize_mem_0_data_0(store_addr)` // call builtins, save content to designated data section
/// - ...
///
/// #### Other Functions
/// - `start`
/// - `function_resolver` // if just-in-time function resolution is enabled.
///
/// #### Materializer Stubs
/// - calling imported/builtin functions
/// - calling indirect functions
///
/// #### The Store Data Section
/// - `builtins -> (length, func_addr*)` // builtin function fixup
/// - `functions -> (length, ((func_addr, (length, type*))*)` // imported fn fixup and indirect calls
/// - `memories -> (length, (memory_addr, min, max)*)` // memory base address fixup
/// - `tables -> (length, (table_addr, min, max)*)` // table base address fixup
/// - `globals -> (length, (global_addr, mut)*)` // global address fixup
///
/// #### Misc
/// - loading important values like memory address into registers from the store data section
#[derive(Debug)]
pub struct LLVM {
    pub context: LLContext,
    pub module: Option<LLModule>,
    pub info: LLVMInfo,
}

/// Compilation information about an LLVM Module.
#[derive(Debug, Default)]
pub struct LLVMInfo {
    pub types: Vec<LLFunctionType>,
}

impl LLVM {
    /// Creates pinned LLVM instance.
    pub fn new() -> Result<Pin<Box<Self>>> {
        // TODO(appcypher): Initialize target, asm printer.

        let mut this = Box::pin(Self {
            context: LLContext::new(),
            module: None,
            info: LLVMInfo::default(),
        });

        // The module field references the context field so this is self-referential.
        this.module = Some(this.context.create_module("initial")?);

        Ok(this)
    }
}

impl Drop for LLVM {
    fn drop(&mut self) {
        // TODO(appcypher): ISSUE:
        // Results in double free of Context.
        // unsafe { LLVMShutdown() }
    }
}
