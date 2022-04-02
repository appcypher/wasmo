use llvm_sys::{
    core::{
        LLVMDoubleTypeInContext, LLVMFloatTypeInContext, LLVMFunctionType, LLVMInt128TypeInContext,
        LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMStructType, LLVMVoidTypeInContext,
    },
    prelude::LLVMTypeRef,
};

use super::context::LLContext;

/// This is based on wasm num and vector types.
pub(crate) enum LLNumTypeKind {
    I32,
    I64,
    I128,
    F32,
    F64,
}

/// Wrapper for LLVM number types (e.g. i64, f32) which is based on wasm num and vector types.
///
/// # Safety
/// Only a kind of each `LLVMTypeRef` is ever created. They are singletons and are never freed.
///
/// - https://llvm.org/doxygen/classllvm_1_1Type.html#details
/// - https://llvm.org/docs/LangRef.html#integer-type
pub(crate) struct LLNumType(LLVMTypeRef);

/// Wrapper for LLVM pointer types (e.g. i64*, [2 x double]*).
///
/// # Safety
/// See [`LLNumType`](struct.LLNumType.html)
///
/// - https://llvm.org/docs/LangRef.html#pointer-type
pub(crate) struct LLPointerType(LLVMTypeRef);

/// Wrapper for LLVM vector types (e.g. <4 x i64>).
///
/// # Safety
/// See [`LLNumType`](struct.LLNumType.html)
///
/// - https://llvm.org/docs/LangRef.html#vector-type
pub(crate) struct LLVectorType(LLVMTypeRef);

/// Wrapper for LLVM array type (e.g. [4 x double]).
///
/// # Safety
/// See [`LLNumType`](struct.LLNumType.html)
///
/// - https://llvm.org/docs/LangRef.html#array-type
pub(crate) struct LLArrayType(LLVMTypeRef);

/// Wrapper for LLVM void type.
///
/// # Safety
/// See [`LLNumType`](struct.LLNumType.html)
///
/// - https://llvm.org/docs/LangRef.html#void-type
pub(crate) struct LLVoidType(LLVMTypeRef);

/// Wrapper for LLVM struct type.
///
/// # Safety
/// Structure types are a bit more complicated than scalar types because we need to allocate the array of types that gets passed to it.
///
/// The good thing however is that LLVM does not depend on our base pointer. It reallocates the params within the LLVM context.
///
/// - https://llvm.org/doxygen/Type_8cpp_source.html#l00361
pub(crate) struct LLStructType(LLVMTypeRef);

/// Wrapper for LLVM function type.
///
/// # Safety
/// Function types are a bit more complicated than scalar types because we need to allocate the array of types that gets passed to it.
///
/// The good thing however is that LLVM does not depend on our base pointer. It reallocates the params within the LLVM context.
///
/// - https://llvm.org/doxygen/Type_8cpp_source.html#l00361
#[derive(Debug)]
pub(crate) struct LLFunctionType(LLVMTypeRef);

/// A limited variants of types that can be returned by an LLVM function
pub(crate) enum LLResultType {
    Void(LLVoidType),
    Num(LLNumType),
    Struct(LLStructType),
}

impl LLNumType {
    /// Creates an LLVM number type.
    ///
    /// # Safety
    /// LLContext does not own type here.
    pub(crate) fn new(context: &LLContext, kind: LLNumTypeKind) -> Self {
        use LLNumTypeKind::*;
        let context_ref = unsafe { context.as_ptr() };
        let type_ref = unsafe {
            match kind {
                I32 => LLVMInt32TypeInContext(context_ref),
                I64 => LLVMInt64TypeInContext(context_ref),
                I128 => LLVMInt128TypeInContext(context_ref),
                F32 => LLVMFloatTypeInContext(context_ref),
                F64 => LLVMDoubleTypeInContext(context_ref),
            }
        };

        Self(type_ref)
    }

    pub(crate) unsafe fn as_ptr(&self) -> LLVMTypeRef {
        self.0
    }
}

impl LLVoidType {
    /// Creates an LLVM void type.
    ///
    /// # Safety
    /// See [`LLNumType`](struct.LLNumType.html)
    pub(crate) fn new(context: &LLContext) -> Self {
        Self(unsafe { LLVMVoidTypeInContext(context.as_ptr()) })
    }

    pub(crate) unsafe fn as_ptr(&self) -> LLVMTypeRef {
        self.0
    }
}

impl LLStructType {
    /// Creates a new LLVM array type.
    ///
    /// # Safety
    /// See [LLStructType](struct.LLStructType.html) for safety.
    pub(crate) fn new(types: &[LLNumType], is_packed: bool) -> Self {
        let types = types
            .iter()
            .map(|p| unsafe { p.as_ptr() })
            .collect::<Vec<_>>();

        Self(unsafe {
            LLVMStructType(
                types.as_ptr() as *mut LLVMTypeRef,
                types.len() as u32,
                is_packed as i32,
            )
        })
    }

    pub(super) unsafe fn as_ptr(&self) -> LLVMTypeRef {
        self.0
    }
}

impl LLFunctionType {
    /// Creates a new LLVM function type.
    /// # Safety
    /// See [LLFunctionType](struct.LLFunctionType.html) for safety.
    pub(crate) fn new(params: &[LLNumType], result: &LLResultType, is_varargs: bool) -> Self {
        let params = params
            .iter()
            .map(|p| unsafe { p.as_ptr() })
            .collect::<Vec<_>>();

        Self(unsafe {
            LLVMFunctionType(
                result.as_ptr(),
                params.as_ptr() as *mut LLVMTypeRef,
                params.len() as u32,
                is_varargs as i32,
            )
        })
    }

    pub(super) unsafe fn as_ptr(&self) -> LLVMTypeRef {
        self.0
    }
}

impl LLResultType {
    pub(crate) unsafe fn as_ptr(&self) -> LLVMTypeRef {
        use LLResultType::*;
        match self {
            Void(v) => v.as_ptr(),
            Num(n) => n.as_ptr(),
            Struct(s) => s.as_ptr(),
        }
    }
}
