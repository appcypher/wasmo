use anyhow::Result;
use std::ffi::CString;

use llvm_sys::{
    core::{LLVMAddFunction, LLVMGetParam},
    prelude::LLVMValueRef,
};

use crate::{basic_block::LLBasicBlock, context::LLContext, impl_trait, types::LLResultType};

use super::{module::LLModule, types::LLFunctionType};

//-----------------------------------------------------------------------------
// Macros
//-----------------------------------------------------------------------------

macro_rules! create_value_struct {
    ($ty:ident, $doc_title:expr) => {
        #[doc = $doc_title]
        /// # Ownership
        /// - Owned by an LLVM Module.
        ///
        /// https://llvm.org/doxygen/classllvm_1_1Value.html#details
        #[derive(Debug, Clone)]
        pub struct $ty(LLVMValueRef);

        impl $ty {
            pub(super) fn from_ptr(ptr: LLVMValueRef) -> Self {
                Self(ptr)
            }

            /// Returns the underlying LLVMValueRef of this value.
            ///
            /// # Safety
            /// - Unsafe because it exposes a raw pointer gotten from LLVM ffi.
            #[allow(unused)]
            pub(crate) unsafe fn as_ptr(&self) -> LLVMValueRef {
                self.0
            }
        }
    };
}

//-----------------------------------------------------------------------------
// Traits
//-----------------------------------------------------------------------------

/// For types that are LLVMValueRef.
pub trait LLValue {
    /// Returns the underlying LLVMValueRef of this value.
    ///
    /// # Safety
    /// - Unsafe because it exposes a raw pointer gotten from LLVM ffi.
    unsafe fn value_ref(&self) -> LLVMValueRef;
}

//------------------------------------------------------------------------------
// Type Definitions
//------------------------------------------------------------------------------

/// LLVM Function wrapper.
///
/// # Safety
/// It is unsafe to use the reference of `LLFunctionType` because its params can be independently freed.
/// Holding an `Rc` to it ensures that that does not happen.
///
/// WARNING: This is safe only if we can only create a Function from a Module.
///
/// # Ownership
/// - Owned by an LLVM Module.
/// - Owns the basic blocks and arguments added to it.
///
/// - https://llvm.org/doxygen/Function_8cpp_source.html#l00409
/// - https://llvm.org/doxygen/Function_8cpp_source.html#l00509
/// - https://llvm.org/doxygen/classllvm_1_1Value.html#details
#[derive(Debug)]
pub struct LLFunction(LLVMValueRef);

create_value_struct! {
    LLParam,
    "Wraps the LLVM param value"
}

create_value_struct! {
    LLUnreachable,
    "Wraps the LLVM unreachable value"
}

create_value_struct! {
    LLAlloca,
    "Wraps the LLVM alloca value"
}

create_value_struct! {
    LLStore,
    "Wraps the LLVM store value"
}

create_value_struct! {
    LLLoad,
    "Wraps the LLVM load value"
}

create_value_struct! {
    LLRet,
    "Wraps the LLVM ret value"
}

create_value_struct! {
    LLRetVoid,
    "Wraps the LLVM ret void value"
}

create_value_struct! {
    LLBr,
    "Wraps the LLVM br value"
}

create_value_struct! {
    LLCondBr,
    "Wraps the LLVM cond br value"
}

create_value_struct! {
    LLAdd,
    "Wraps the LLVM add value"
}

create_value_struct! {
    LLSub,
    "Wraps the LLVM sub value"
}

create_value_struct! {
    LLConstStruct,
    "Wraps the LLVM const struct value"
}

create_value_struct! {
    LLConstInt,
    "Wraps the LLVM const int value"
}

//------------------------------------------------------------------------------
// Implementations
//------------------------------------------------------------------------------

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
    pub(super) fn new(
        name: &str,
        module: &mut LLModule,
        function_type: &LLFunctionType,
    ) -> Result<Self> {
        Ok(Self(unsafe {
            LLVMAddFunction(
                module.as_ptr(),
                CString::new(name)?.as_ptr(),
                function_type.as_ptr(),
            )
        }))
    }

    /// Creates a new LLVM Basic Block.
    pub fn create_basic_block(&self, name: &str, context: &LLContext) -> Result<LLBasicBlock> {
        LLBasicBlock::new(name, self, context)
    }

    /// Gets the param at the given index.
    pub fn get_param(&self, index: u32) -> LLParam {
        LLParam::from_ptr(unsafe { LLVMGetParam(self.as_ptr(), index) })
    }

    pub(crate) unsafe fn as_ptr(&self) -> LLVMValueRef {
        self.0
    }
}

impl_trait! {
    LLValue(value_ref -> LLVMValueRef) for {
        LLFunction,
        LLParam,
        LLUnreachable,
        LLAlloca,
        LLStore,
        LLLoad,
        LLRet,
        LLRetVoid,
        LLBr,
        LLCondBr,
        LLAdd,
        LLSub,
        LLConstStruct,
        LLConstInt,
    }
}
