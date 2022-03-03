// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use std::pin::Pin;

use llvm_sys::{
    core::{
        LLVMContextCreate, LLVMContextDispose, LLVMDoubleTypeInContext, LLVMFloatTypeInContext,
        LLVMInt32TypeInContext, LLVMInt64TypeInContext,
    },
    prelude::LLVMContextRef,
};

use super::types::{LLFunctionType, LLType, LLTypeRef};

/// This a wrapper for LLVM Context.
#[derive(Debug)]
pub(crate) struct LLContext {
    context_ref: LLVMContextRef,
}

impl LLContext {
    pub(crate) fn new() -> Self {
        Self {
            context_ref: unsafe { LLVMContextCreate() },
        }
    }

    pub(crate) fn as_ptr(&self) -> LLVMContextRef {
        self.context_ref
    }

    pub(crate) fn i64_type(&self) -> LLType {
        LLType::I64(LLTypeRef::new(unsafe {
            LLVMInt64TypeInContext(self.context_ref)
        }))
    }

    pub(crate) fn i32_type(&self) -> LLType {
        LLType::I64(LLTypeRef::new(unsafe {
            LLVMInt32TypeInContext(self.context_ref)
        }))
    }

    pub(crate) fn f64_type(&self) -> LLType {
        LLType::I64(LLTypeRef::new(unsafe {
            LLVMDoubleTypeInContext(self.context_ref)
        }))
    }

    pub(crate) fn f32_type(&self) -> LLType {
        LLType::I64(LLTypeRef::new(unsafe {
            LLVMFloatTypeInContext(self.context_ref)
        }))
    }

    pub(crate) fn function_type<'t>(
        &self,
        params: &'t [LLType],
        result: &'t LLType,
        is_varargs: bool,
    ) -> Pin<Box<LLFunctionType<'t>>> {
        LLFunctionType::new(params, result, is_varargs)
    }
}

impl Drop for LLContext {
    fn drop(&mut self) {
        // Dispose of the LLVM context.
        unsafe {
            LLVMContextDispose(self.context_ref);
        }
    }
}
