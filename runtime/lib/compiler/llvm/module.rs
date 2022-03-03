// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use std::pin::Pin;

use utilities::result::Result;

use llvm_sys::{
    core::{LLVMDumpModule, LLVMModuleCreateWithNameInContext},
    prelude::{LLVMContextRef, LLVMModuleRef},
};

use super::{function::LLFunction, types::LLFunctionType, utils::LLString};

/// A wrapper for LLVM Module.
///
/// # Safety
///
/// When a Module references a Context, the Context frees it when it gets dropped.
///
/// We leverage this behavior by not disposing the Module explicitly on drop, letting associated Context do the job.
/// This is safe because we can only create a Module from a Context.
///
/// NOTE: We can't use lifetime parameter since it leads to unresolvable self-referential structs when an `LLModule` is stored in the same struct as the associated `LLContext`.
///
/// https://lists.llvm.org/pipermail/llvm-dev/2018-September/126134.html
#[derive(Debug)]
pub(crate) struct LLModule {
    name: Pin<Box<LLString>>,
    module_ref: LLVMModuleRef,
}

impl LLModule {
    /// This is the only way to create an LLModule to ensure it has an associated Context.
    pub(crate) fn new(name: Pin<Box<LLString>>, context: LLVMContextRef) -> Result<Pin<Box<Self>>> {
        let mut this = Self {
            name,
            module_ref: std::ptr::null_mut(),
        };

        this.module_ref = unsafe { LLVMModuleCreateWithNameInContext(this.name.as_ptr(), context) };

        Ok(Box::pin(this))
    }

    pub(crate) fn add_function<'m, 's, 't>(
        &'m self,
        name: &'s LLString,
        signature: &'t LLFunctionType,
    ) -> Result<LLFunction<'m, 's, 't>> {
        LLFunction::attach(self, name, signature)
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
