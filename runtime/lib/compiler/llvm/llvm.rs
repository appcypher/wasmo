use std::pin::Pin;

use super::{context::LLContext, module::LLModule};
use anyhow::Result;
use llvm_sys::core::LLVMShutdown;

/// Converts WebAssembly semantics to LLVM code, handles materialization.
///
/// # Safety
/// This type is self-referential. We can only construct it as a pinned object.
/// This prevents pointer issues that comes with moving the object.
///
/// The codegen phase generates some boilerplate code that is used during initialization, and execution.
/// They include:
///
/// #### The Resolvers Functions
/// - `add_imported_function_resolver(resolver_addr)` -> save address to designated data section
/// - `resolve_imported_memories(store_ref_addr)` -> call intrinsics, save address to designated data section
/// - `resolve_imported_tables(store_ref_addr)` -> call intrinsics, save address to designated data section
/// - `resolve_imported_globals(store_ref_addr)` -> call intrinsics, save address to designated data section
///
/// #### The Initializer Functions
/// - `initialize_local_memories(store_ref_addr)` -> call intrinsics, save address to designated data section
/// - `initialize_local_tables(store_ref_addr)` -> call intrinsics, save address to designated data section
/// - `initialize_local_globals(store_ref_addr)` -> call intrinsics, save content to designated data section
///
/// #### The Start Function
/// - `_start`
///
/// #### Stubs
/// - imported functions
/// - calling indirect functions
///
/// #### The Data Section
/// - memories -> (length, memory_addr*)
/// - tables -> (length, table_addr*)
/// - globals -> (length, global_addr*)
/// - functions -> (length, (type, func_addr)*)
/// - imported_function_resolver -> returns the resolved func_addr
///
/// #### Misc
/// - loading important values like memory address into registers
#[derive(Debug)]
pub(crate) struct LLVM {
    pub(crate) context: LLContext,
    pub(crate) module: Option<LLModule>,
}

impl LLVM {
    /// Creates pinned LLVM instance.
    pub(crate) fn new() -> Result<Pin<Box<Self>>> {
        // TODO(appcypher): Initialize target, asm printer.

        let mut this = Box::pin(Self {
            context: LLContext::new(),
            module: None,
        });

        // The module field references the context field so this is self-referential.
        this.module = Some(LLModule::new("initial", &this.context)?);

        Ok(this)
    }
}

impl Drop for LLVM {
    fn drop(&mut self) {
        unsafe { LLVMShutdown() }
    }
}
