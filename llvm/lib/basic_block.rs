use std::ffi::CString;

use anyhow::Result;
use llvm_sys::{core::LLVMAppendBasicBlockInContext, prelude::LLVMBasicBlockRef};

use crate::{context::LLContext, values::LLFunction};

/// LLVM BasicBlock wrapper.
///
/// # Ownership
/// - Owned by an LLVM Function.
///
/// # References
/// - https://llvm.org/doxygen/Function_8cpp_source.html#l00509
pub struct LLBasicBlock(LLVMBasicBlockRef);

impl LLBasicBlock {
    /// Creates a new LLVM BasicBlock.
    ///
    /// This is the only way to create a LLBasicBlock, ensuring it has an associated Function.
    /// Therefore a Function must already exist to dispose it.
    ///
    /// # Safety
    /// - BasicBlock can only be created from a Function that frees it.
    ///
    /// ### References
    /// - https://llvm.org/doxygen/Function_8cpp_source.html#l00509
    pub(super) fn new(name: &str, function: &LLFunction, context: &LLContext) -> Result<Self> {
        Ok(Self(unsafe {
            LLVMAppendBasicBlockInContext(
                context.as_ptr(),
                function.as_ptr(),
                CString::new(name)?.as_ptr(),
            )
        }))
    }

    pub(crate) unsafe fn as_ptr(&self) -> LLVMBasicBlockRef {
        self.0
    }
}
