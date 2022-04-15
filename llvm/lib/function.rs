use anyhow::Result;
use std::ffi::CString;

use llvm_sys::{core::LLVMAddFunction, prelude::LLVMValueRef};

use super::{module::LLModule, types::LLFunctionType};

/// LLVM Function wrapper.
///
/// # Safety
/// It is unsafe to use the reference of `LLFunctionType` because its params can be independently freed.
/// Holding an `Rc` to it ensures that that does not happen.
///
/// WARNING: This is safe only if we can only create a Function from a Module.
///
/// # Ownership
/// - Owned by LLVM Module.
/// - Owns the basic blocks and arguments added to it.
///
/// - https://llvm.org/doxygen/Function_8cpp_source.html#l00409
/// - https://llvm.org/doxygen/Function_8cpp_source.html#l00509
#[derive(Debug)]
pub struct LLFunction(LLVMValueRef);

impl LLFunction {
    /// Creates a new LLVM Function.
    ///
    /// This is the only way to create an LLFunction, ensuring it has an associated Module.
    /// Therefore a Module must already exist to dispose it.
    ///
    /// # Safety
    /// - Module can only be created from a Context that frees it.
    /// - TODO(appcypher): It is unclear if the LLVM depends on `name` pointer.
    ///
    /// ### References
    /// - https://llvm.org/doxygen/Twine_8h_source.html#l00271
    /// - https://llvm.org/doxygen/Twine_8h_source.html#l00477
    /// - https://llvm.org/doxygen/Value_8cpp_source.html#l00315
    /// - https://llvm.org/doxygen/StringRef_8h_source.html#l00107
    pub fn new(name: &str, module: &mut LLModule, function_type: LLFunctionType) -> Result<Self> {
        Ok(Self(unsafe {
            LLVMAddFunction(
                module.as_ptr(),
                CString::new(name)?.as_ptr(),
                function_type.as_ptr(),
            )
        }))
    }

    pub(crate) unsafe fn as_ptr(&self) -> LLVMValueRef {
        self.0
    }
}
