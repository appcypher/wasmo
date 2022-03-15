use std::rc::Rc;

use anyhow::Result;
use llvm_sys::{
    core::{LLVMContextCreate, LLVMContextDispose},
    prelude::LLVMContextRef,
};

use super::{
    module::LLModule,
    types::{LLFunctionType, LLType, LLTypeKind},
};

/// This a wrapper for LLVM Context.
///
/// # Ownership
/// Owns the LLVM Module.
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

    pub(crate) fn create_module(&self, name: &str) -> Result<LLModule> {
        LLModule::new(name, self)
    }

    pub(crate) fn as_ptr(&self) -> LLVMContextRef {
        self.context_ref
    }

    pub(crate) fn i64_type(&self) -> LLType {
        LLType::new(self, LLTypeKind::I64)
    }

    pub(crate) fn i32_type(&self) -> LLType {
        LLType::new(self, LLTypeKind::I32)
    }

    pub(crate) fn f64_type(&self) -> LLType {
        LLType::new(self, LLTypeKind::F64)
    }

    pub(crate) fn f32_type(&self) -> LLType {
        LLType::new(self, LLTypeKind::F32)
    }

    pub(crate) fn function_type(
        &self,
        params: &[LLType],
        result: &LLType,
        is_varargs: bool,
    ) -> Rc<LLFunctionType> {
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
