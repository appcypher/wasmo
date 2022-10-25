use std::pin::Pin;

use anyhow::Result;

use super::{context::LLContext, module::LLModule, types::LLFunctionType};
// use llvm_sys::core::LLVMShutdown;

/// The LLVM wrapper.
///
/// # Safety
/// This type is self-referential so we can only construct it as a pinned object.
/// This prevents pointer issues that comes with moving the object.
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
