
use std::rc::Rc;

use llvm_sys::{
    core::{
        LLVMDoubleTypeInContext, LLVMFloatTypeInContext, LLVMFunctionType, LLVMInt32TypeInContext,
        LLVMInt64TypeInContext,
    },
    prelude::LLVMTypeRef,
};

use super::context::LLContext;

pub(crate) enum LLTypeKind {
    I64,
    I32,
    F64,
    F32,
}

/// This is a wrapper for LLVM Type.
///
/// # Safety
/// Only a kind of each `LLVMTypeRef` is ever created. They are singletons and are never freed.
///
/// - https://llvm.org/doxygen/classllvm_1_1Type.html#details
pub(crate) struct LLType {
    type_ref: LLVMTypeRef,
    kind: LLTypeKind,
}

impl LLType {
    /// Creates a new LLVM type.
    ///
    /// # Safety
    /// LLContext does not own type here.
    pub(crate) fn new(context: &LLContext, kind: LLTypeKind) -> Self {
        use LLTypeKind::*;
        let context_ref = context.as_ptr();
        let type_ref = unsafe {
            match kind {
                I64 => LLVMInt64TypeInContext(context_ref),
                I32 => LLVMInt32TypeInContext(context_ref),
                F64 => LLVMDoubleTypeInContext(context_ref),
                F32 => LLVMFloatTypeInContext(context_ref),
            }
        };

        Self { type_ref, kind }
    }

    pub(crate) fn as_ptr(&self) -> LLVMTypeRef {
        self.type_ref
    }
}

/// This is a wrapper for LLVM Function Type.
///
/// # Safety
/// As mentioned [here](struct.LLType.html), `LLVMTypeRef`s are singletons and never freed.
/// However function types need pointers to some array of types as params and it is up to the owner of the array to free it.
///
/// This struct owns its params array and it is safe for params to hold pointers to non-function `LLVMTypeRef`s since they are never freed.
///
/// WARNING:
/// It is unsafe to use the function reference because the params can be dropped when this struct goes out of scope.
/// It is important to make sure the function is not dropped before the dependents.
#[derive(Debug)]
pub(crate) struct LLFunctionType {
    params: Vec<LLVMTypeRef>,
    result: LLVMTypeRef,
    is_varargs: i32,
}

impl LLFunctionType {
    /// Creates a new LLVM function type.
    ///
    /// # Safety
    /// See [LLFunctionType](struct.LLFunctionType.html) for safety.
    ///
    /// `Rc` helps with safety here because dependents can rely on reference counting.
    pub(crate) fn new(params: &[LLType], result: &LLType, is_varargs: bool) -> Rc<Self> {
        Rc::new(Self {
            params: params.iter().map(|p| p.as_ptr()).collect::<Vec<_>>(),
            result: result.as_ptr(),
            is_varargs: is_varargs as i32,
        })
    }

    /// Returns the pointer behind the function.
    ///
    /// # Safety
    ///
    /// WARNING:
    /// It is unsafe to use the function reference because the params can be dropped when this struct goes out of scope.
    /// It is important to make sure the function is not dropped before the dependents.
    pub(super) unsafe fn as_ptr(&self) -> LLVMTypeRef {
        LLVMFunctionType(
            self.result,
            self.params.as_ptr() as *mut LLVMTypeRef,
            self.params.len() as u32,
            self.is_varargs,
        )
    }
}
