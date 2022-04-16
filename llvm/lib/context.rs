use anyhow::Result;
use llvm_sys::{
    core::{LLVMContextCreate, LLVMContextDispose},
    prelude::LLVMContextRef,
};

use super::{
    module::LLModule,
    types::{LLFunctionType, LLNumType, LLNumTypeKind, LLResultType, LLStructType, LLVoidType},
};

/// LLVM Context wrapper.
///
/// # Ownership
/// - Owns the LLVM Module.
/// - Not owned by anything.
///
/// ### References
/// - https://llvm.org/doxygen/Module_8cpp_source.html#l00079
/// - https://llvm.org/doxygen/LLVMContextImpl_8cpp_source.html#l00056
#[derive(Debug)]
pub struct LLContext(LLVMContextRef);

impl LLContext {
    pub fn new() -> Self {
        Self(unsafe { LLVMContextCreate() })
    }

    pub fn create_module(&self, name: &str) -> Result<LLModule> {
        LLModule::new(name, self)
    }

    pub(crate) unsafe fn as_ptr(&self) -> LLVMContextRef {
        self.0
    }

    pub fn i32_type(&self) -> LLNumType {
        LLNumType::new(self, LLNumTypeKind::I32)
    }

    pub fn i64_type(&self) -> LLNumType {
        LLNumType::new(self, LLNumTypeKind::I64)
    }

    pub fn i128_type(&self) -> LLNumType {
        LLNumType::new(self, LLNumTypeKind::I128)
    }

    pub fn f32_type(&self) -> LLNumType {
        LLNumType::new(self, LLNumTypeKind::F32)
    }

    pub fn f64_type(&self) -> LLNumType {
        LLNumType::new(self, LLNumTypeKind::F64)
    }

    pub fn void_type(&self) -> LLVoidType {
        LLVoidType::new(self)
    }

    pub fn struct_type(&self, types: &[LLNumType], is_packed: bool) -> LLStructType {
        LLStructType::new(types, is_packed)
    }

    pub fn function_type(
        &self,
        params: &[LLNumType],
        result: &LLResultType,
        is_varargs: bool,
    ) -> LLFunctionType {
        LLFunctionType::new(params, result, is_varargs)
    }
}

impl Default for LLContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LLContext {
    fn drop(&mut self) {
        // Dispose of the LLVM context.
        unsafe {
            LLVMContextDispose(self.0);
        }
    }
}
