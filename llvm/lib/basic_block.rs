use std::ffi::CString;

use anyhow::Result;
use llvm_sys::{
    core::{
        LLVMAppendBasicBlockInContext, LLVMAppendExistingBasicBlock, LLVMCreateBasicBlockInContext,
        LLVMDeleteBasicBlock,
    },
    prelude::LLVMBasicBlockRef,
};

use crate::{context::LLContext, not_null, values::LLFunction};

/// LLVM BasicBlock wrapper.
///
/// # Ownership
/// - Owned by an LLVM Function if appended.
///
/// # References
/// - https://llvm.org/doxygen/Function_8cpp_source.html#l00509
pub struct LLBasicBlock {
    ptr: LLVMBasicBlockRef,
    is_appended: bool,
}

impl LLBasicBlock {
    /// Creates a new LLBasicBlock.
    pub fn new(name: &str, context: &LLContext) -> Result<Self> {
        Ok(Self {
            ptr: unsafe {
                LLVMCreateBasicBlockInContext(context.as_ptr(), CString::new(name)?.as_ptr())
            },
            is_appended: false,
        })
    }

    /// Appends a LLBasicBlock to the end of specified function.
    pub(super) fn append_to_function(&mut self, function: &LLFunction) {
        unsafe { LLVMAppendExistingBasicBlock(function.as_ptr(), self.ptr) }
        self.is_appended = true;
    }

    /// Creates a new LLVM BasicBlock and appends it to a function at the same time.
    pub(super) fn create_and_append(
        name: &str,
        function: &LLFunction,
        context: &LLContext,
    ) -> Result<Self> {
        Ok(Self {
            ptr: unsafe {
                not_null!(LLVMAppendBasicBlockInContext(
                    context.as_ptr(),
                    function.as_ptr(),
                    CString::new(name)?.as_ptr(),
                ))
            },
            is_appended: true,
        })
    }

    pub(crate) unsafe fn as_ptr(&self) -> LLVMBasicBlockRef {
        self.ptr
    }
}

impl Drop for LLBasicBlock {
    fn drop(&mut self) {
        if !self.is_appended {
            // TODO(appcypher): PROBLEM:
            // Commenting this out makes the module dump stop hanging but it also leads to dangling pointer.
            // Elaborate RC and ugly pinning?
            // Check how inkwell handles this.
            // unsafe { LLVMDeleteBasicBlock(self.ptr) }
        }
    }
}
