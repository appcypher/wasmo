// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use anyhow::Result;
use std::{ffi::CString, rc::Rc};

use llvm_sys::{core::LLVMAddFunction, prelude::LLVMValueRef};

use super::{module::LLModule, types::LLFunctionType};

/// This is a wrapper for LLVM Function.
///
/// # Safety
/// It is unsafe to use the reference of `LLFunctionType` because its params can be independently freed.
/// Holding an `Rc` to it ensures that that does not happen.
pub(crate) struct LLFunction {
    function_ref: LLVMValueRef,
    function_type: Rc<LLFunctionType>,
}

impl LLFunction {
    /// Creates a new LLVM function.
    ///
    /// # Safety
    /// The use of temporary `CString` expecting that value will be copied in `LLVMAddFunction` might not be safe.
    ///
    /// - https://llvm.org/doxygen/Twine_8h_source.html#l00477
    /// - https://llvm.org/doxygen/Value_8cpp_source.html#l00315
    pub(crate) fn new(
        name: &str,
        module: &LLModule,
        function_type: Rc<LLFunctionType>,
    ) -> Result<Self> {
        Ok(Self {
            function_ref: unsafe {
                LLVMAddFunction(
                    module.as_ptr(),
                    CString::new(name)?.as_ptr(),
                    function_type.as_ptr(),
                )
            },
            function_type,
        })
    }
}
