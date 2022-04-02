use std::pin::Pin;

use super::{context::LLContext, module::LLModule, types::LLFunctionType};
use anyhow::Result;
use llvm_sys::core::LLVMShutdown;

/// Converts WebAssembly semantics to LLVM code and handles materialization.
///
/// # Safety
/// This type is self-referential. We can only construct it as a pinned object.
/// This prevents pointer issues that comes with moving the object.
///
/// The codegen phase generates some boilerplate code that is used during initialization, and execution.
/// They include:
///
/// #### The Resolvers Functions
/// - `add_function_resolver(resolver_addr)` // save address to designated data section
/// - `resolve_imported_memories(store_ref_addr)` // call intrinsics, save address to designated data section
/// - `resolve_imported_tables(store_ref_addr)` // call intrinsics, save address to designated data section
/// - `resolve_imported_globals(store_ref_addr)` // call intrinsics, save address to designated data section
///
/// #### The Local Setup Functions
/// - `setup_local_memories(store_ref_addr)` // call intrinsics, call initializers, save address to designated data section
/// - `setup_local_tables(store_ref_addr)` // call intrinsics, call initializers, save address to designated data section
/// - `setup_local_globals(store_ref_addr)` // call intrinsics, call initializers, save content to designated data section
///
/// #### The Initializer Functions
/// - `initialize_mem_0_data_0(store_ref_addr)` // call intrinsics, save content to designated data section
/// - ...
///
/// #### The Start Function
/// - `_start`
///
/// #### Materializer Stubs
/// - calling imported functions
/// - calling indirect functions
///
/// #### The Store Data Section
/// #### The Store Data Section
/// - `function_resolver -> func_addr` // resolves intrinsics and imported functions
/// - `intrinsics -> (length, func_addr*)` // intrinsic function fixup
/// - `functions -> (length, (type, func_addr)*)` // imported fn fixup and indirect calls
/// - `memories -> (length, memory_base_addr*)` // memory base address fixup
/// - `tables -> (length, table_base_addr*)` // table base address fixup
/// - `globals -> (length, global_addr*)` // global address fixup
///
/// #### Misc
/// - loading important values like memory address into registers from the store data section
#[derive(Debug)]
pub(crate) struct LLVM {
    pub(crate) context: LLContext,
    pub(crate) module: Option<LLModule>,
    pub(crate) info: LLVMInfo,
}

/// Compilation information about an LLVM Module.
#[derive(Debug, Default)]
pub(crate) struct LLVMInfo {
    types: Vec<LLFunctionType>,
}

impl LLVM {
    /// Creates pinned LLVM instance.
    pub(crate) fn new() -> Result<Pin<Box<Self>>> {
        // TODO(appcypher): Initialize target, asm printer.

        let mut this = Box::pin(Self {
            context: LLContext::new(),
            module: None,
            info: LLVMInfo::default(),
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
