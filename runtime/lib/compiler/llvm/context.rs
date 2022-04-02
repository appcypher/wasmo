use anyhow::Result;
use llvm_sys::{
    core::{LLVMContextCreate, LLVMContextDispose},
    prelude::LLVMContextRef,
};

use super::{
    module::LLModule,
    types::{LLFunctionType, LLNumType, LLNumTypeKind, LLResultType, LLStructType, LLVoidType},
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

    pub(crate) unsafe fn as_ptr(&self) -> LLVMContextRef {
        self.context_ref
    }

    pub(crate) fn i32_type(&self) -> LLNumType {
        LLNumType::new(self, LLNumTypeKind::I32)
    }

    pub(crate) fn i64_type(&self) -> LLNumType {
        LLNumType::new(self, LLNumTypeKind::I64)
    }

    pub(crate) fn i128_type(&self) -> LLNumType {
        LLNumType::new(self, LLNumTypeKind::I128)
    }

    pub(crate) fn f32_type(&self) -> LLNumType {
        LLNumType::new(self, LLNumTypeKind::F32)
    }

    pub(crate) fn f64_type(&self) -> LLNumType {
        LLNumType::new(self, LLNumTypeKind::F64)
    }

    pub(crate) fn void_type(&self) -> LLVoidType {
        LLVoidType::new(self)
    }

    pub(crate) fn struct_type(&self, types: &[LLNumType], is_packed: bool) -> LLStructType {
        LLStructType::new(types, is_packed)
    }

    pub(crate) fn function_type(
        &self,
        params: &[LLNumType],
        result: &LLResultType,
        is_varargs: bool,
    ) -> LLFunctionType {
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
