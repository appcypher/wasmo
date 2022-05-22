use std::ffi::CString;

use anyhow::Result;

use llvm_sys::{
    core::{LLVMDumpModule, LLVMModuleCreateWithNameInContext},
    prelude::LLVMModuleRef,
};

use crate::{types::LLFunctionType, values::LLFunction, not_null};

use super::context::LLContext;

/// LLVM Module wrapper.
///
/// # Safety
///
/// When a Module references a Context, the Context frees it when it gets dropped.
///
/// We leverage this behavior by not disposing the Module explicitly on drop, letting associated Context do the job.
///
/// ### References
/// - https://lists.llvm.org/pipermail/llvm-dev/2018-September/126134.html
/// - https://llvm.org/doxygen/Module_8cpp_source.html#l00079
/// - https://llvm.org/doxygen/LLVMContextImpl_8cpp_source.html#l00052
///
/// # Ownership
/// - Owned by an LLVM Context.
/// - Owns the functions and globals added to it.
///
/// ### References
/// - https://llvm.org/doxygen/Module_8cpp_source.html#l00079
/// - https://llvm.org/doxygen/LLVMContextImpl_8cpp_source.html#l00056
#[derive(Debug)]
pub struct LLModule(LLVMModuleRef);

impl LLModule {
    /// Creates a new LLVM Module.
    ///
    /// This is the only way to create an LLModule, ensuring it has an associated Context.
    /// Therefore a Context must already exist to dispose it.
    ///
    /// # Safety
    /// - Module can only be created from a Context that frees it.
    /// - A temporary `CString` name is safe to use here because it is copied into the LLVM Module.
    ///
    /// ### References
    ///  - https://llvm.org/doxygen/Module_8cpp_source.html#l00072
    pub(super) fn new(name: &str, context: &LLContext) -> Result<Self> {
        Ok(Self(unsafe {
            not_null!(LLVMModuleCreateWithNameInContext(
                CString::new(name)?.as_ptr(),
                context.as_ptr()
            ))
        }))
    }

    pub fn add_function(
        &mut self,
        name: &str,
        function_type: &LLFunctionType,
    ) -> Result<LLFunction> {
        LLFunction::new(name, self, function_type)
    }

    pub(crate) unsafe fn as_ptr(&self) -> LLVMModuleRef {
        self.0
    }

    pub fn print(&self) {
        unsafe {
            LLVMDumpModule(self.0);
        }
    }
}
