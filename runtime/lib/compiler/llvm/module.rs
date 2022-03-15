
use std::{ffi::CString, rc::Rc};

use anyhow::Result;

use llvm_sys::{
    core::{LLVMDumpModule, LLVMModuleCreateWithNameInContext},
    prelude::LLVMModuleRef,
};

use super::{context::LLContext, function::LLFunction, types::LLFunctionType};

/// A wrapper for LLVM Module.
///
/// # Safety
///
/// When a Module references a Context, the Context frees it when it gets dropped.
///
/// We leverage this behavior by not disposing the Module explicitly on drop, letting associated Context do the job.
///
/// WARNING: This is safe only if we can only create a Module from a Context.
///
/// NOTE: We can't use lifetime parameter since it leads to unresolvable self-referential structs when an `LLModule` is stored in the same struct as the associated `LLContext`.
///
/// - https://lists.llvm.org/pipermail/llvm-dev/2018-September/126134.html
/// - https://llvm.org/doxygen/Module_8cpp_source.html#l00079
/// - https://llvm.org/doxygen/LLVMContextImpl_8cpp_source.html#l00052
///
/// # Ownership
/// ???
///
#[derive(Debug)]
pub(crate) struct LLModule {
    module_ref: LLVMModuleRef,
}

impl LLModule {
    /// This is the only way to create an LLModule to ensure it has an associated Context that can dispose it.
    ///
    /// # Safety
    /// A temporary `CString` name is safe to use here because it is copied into the LLVM Module.
    ///
    /// - https://llvm.org/doxygen/Module_8cpp_source.html#l00072
    pub(crate) fn new(name: &str, context: &LLContext) -> Result<Self> {
        Ok(Self {
            module_ref: unsafe {
                LLVMModuleCreateWithNameInContext(CString::new(name)?.as_ptr(), context.as_ptr())
            },
        })
    }

    /// Adds a function to the module.
    ///
    /// # Safety
    /// TODO(appcypher): Investigate safety properly.
    pub(crate) fn add_function(
        &self,
        name: &str,
        signature: Rc<LLFunctionType>,
    ) -> Result<LLFunction> {
        LLFunction::new(name, self, signature)
    }

    pub(crate) fn as_ptr(&self) -> LLVMModuleRef {
        self.module_ref
    }

    pub(crate) fn print(&self) {
        unsafe {
            LLVMDumpModule(self.module_ref);
        }
    }
}