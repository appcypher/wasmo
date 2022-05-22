use anyhow::Result;
use llvm_sys::{
    core::{LLVMContextCreate, LLVMContextDispose},
    prelude::LLVMContextRef,
};

use super::{
    module::LLModule,
    types::{LLFunctionType, LLNumType, LLResultType, LLStructType, LLVoidType},
};
use crate::{
    builder::LLBuilder,
    not_null,
    types::{LLFloat32Type, LLFloat64Type, LLInt128Type, LLInt32Type, LLInt64Type},
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
    /// Creates a new LLVM context.
    pub fn new() -> Self {
        Self(unsafe { not_null!(LLVMContextCreate()) })
    }

    /// Creates a new LLVM module.
    pub fn create_module(&self, name: &str) -> Result<LLModule> {
        LLModule::new(name, self)
    }

    /// Creates a new LLVM builder.
    pub fn create_builder(&self) -> LLBuilder {
        LLBuilder::new(self)
    }

    /// Gets or creates an LLVM i32 type.
    pub fn i32_type(&self) -> LLInt32Type {
        LLInt32Type::new(self)
    }

    /// Gets or creates an LLVM i64 type.
    pub fn i64_type(&self) -> LLInt64Type {
        LLInt64Type::new(self)
    }

    /// Gets or creates an LLVM i128 type.
    pub fn i128_type(&self) -> LLInt128Type {
        LLInt128Type::new(self)
    }

    /// Gets or creates an LLVM float type.
    pub fn f32_type(&self) -> LLFloat32Type {
        LLFloat32Type::new(self)
    }

    /// Gets or creates an LLVM double type.
    pub fn f64_type(&self) -> LLFloat64Type {
        LLFloat64Type::new(self)
    }

    /// Gets or creates an LLVM void type.
    pub fn void_type(&self) -> LLVoidType {
        LLVoidType::new(self)
    }

    /// Gets or creates an LLVM struct type.
    pub fn struct_type(&self, types: &[Box<dyn LLNumType>], is_packed: bool) -> LLStructType {
        LLStructType::new(types, is_packed)
    }

    /// Gets or creates an LLVM function type.
    pub fn function_type(
        &self,
        params: &[Box<dyn LLNumType>],
        result: &dyn LLResultType,
        is_varargs: bool,
    ) -> LLFunctionType {
        LLFunctionType::new(params, result, is_varargs)
    }

    pub(crate) unsafe fn as_ptr(&self) -> LLVMContextRef {
        self.0
    }
}

impl Default for LLContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LLContext {
    fn drop(&mut self) {
        // TODO(appcypher): PROBLEM:
        // Commenting this out makes the module dump stop hanging but it also leads to dangling pointer.
        // Elaborate RC and ugly pinning?
        // Check how inkwell handles this.
        // Dispose of the LLVM context.
        // unsafe {
        //     LLVMContextDispose(self.0);
        // }
    }
}
