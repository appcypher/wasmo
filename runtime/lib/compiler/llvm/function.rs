use anyhow::Result;
use std::{ffi::CString, rc::Rc};

use llvm_sys::{core::LLVMAddFunction, prelude::LLVMValueRef};

use super::{module::LLModule, types::LLFunctionType};

/// This is a wrapper for LLVM Function.
///
/// # Safety
/// It is unsafe to use the reference of `LLFunctionType` because its params can be independently freed.
/// Holding an `Rc` to it ensures that that does not happen.
///
/// WARNING: This is safe only if we can only create a Function from a Module.
///
/// # Ownership
/// Owns the basic blocks and arguments added to it.
///
/// - https://llvm.org/doxygen/Function_8cpp_source.html#l00409
/// - https://llvm.org/doxygen/Function_8cpp_source.html#l00509
#[derive(Debug)]
pub(crate) struct LLFunction {
    function_ref: LLVMValueRef,
    function_type: Rc<LLFunctionType>,
}

impl LLFunction {
    /// Creates a new LLVM function.
    ///
    /// This is the only way to create an LLFunction to ensure it has an associated Module that can dispose it.
    ///
    /// # Safety
    /// Looks like a pointer to the `CString` is held here.
    ///
    /// - https://llvm.org/doxygen/Twine_8h_source.html#l00271
    /// - https://llvm.org/doxygen/Twine_8h_source.html#l00477
    /// - https://llvm.org/doxygen/Value_8cpp_source.html#l00315
    /// - https://llvm.org/doxygen/StringRef_8h_source.html#l00107
    ///
    /// A function is owned by a module but we also need to mutate function and since we cannot create a function without its module,
    /// We decided to create an `Rc` of the function that can be shared. This simplifies what would have turned out tbe a lifetime hell.
    pub(crate) fn new(
        name: &str,
        module: &mut LLModule,
        function_type: Rc<LLFunctionType>,
    ) -> Result<Rc<Self>> {
        let function = Rc::new(Self {
            function_ref: unsafe {
                LLVMAddFunction(
                    module.as_ptr(),
                    CString::new(name)?.as_ptr(),
                    function_type.as_ptr(),
                )
            },
            function_type,
        });

        module.add_function(Rc::clone(&function));

        Ok(function)
    }
}
