use std::ffi::CString;

use anyhow::Result;

use hashbrown::HashMap;
use llvm_sys::{
    core::{LLVMAddFunction, LLVMDumpModule, LLVMModuleCreateWithNameInContext},
    prelude::LLVMModuleRef,
};

use crate::{intrinsics::Intrinsic, not_null, types::LLFunctionType, values::LLFunction};

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
pub struct LLModule {
    ptr: LLVMModuleRef,
    intrinsics: HashMap<&'static str, LLFunction>,
}

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
        Ok(Self {
            ptr: unsafe {
                not_null!(LLVMModuleCreateWithNameInContext(
                    CString::new(name)?.as_ptr(),
                    context.as_ptr()
                ))
            },
            intrinsics: Default::default(),
        })
    }

    pub fn add_function(
        &mut self,
        name: &str,
        function_type: &LLFunctionType,
    ) -> Result<LLFunction> {
        LLFunction::new(name, self, function_type)
    }

    pub fn add_or_get_intrinsic_function(&mut self, intrinsic: &Intrinsic) -> Result<&LLFunction> {
        let name = intrinsic.name;
        // TODO(appcypher): This is suboptimal because it gets twice when the function exists but the alternative does
        // not work either because the compiler complains about confusing a mutable/immutable borrow overlap.
        if self.intrinsics.get(name).is_none() {
            let function = LLFunction::from_ptr(unsafe {
                not_null!(LLVMAddFunction(
                    self.ptr,
                    CString::new(name)?.as_ptr(),
                    (intrinsic.get_type)(),
                ))
            });

            self.intrinsics.insert(name, function);
        }

        Ok(self.intrinsics.get(name).unwrap())
    }

    pub(crate) unsafe fn as_ptr(&self) -> LLVMModuleRef {
        self.ptr
    }

    pub fn print(&self) {
        unsafe {
            LLVMDumpModule(self.ptr);
        }
    }
}
