// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use super::context::LLContext;
use llvm_sys::{core::LLVMFunctionType, prelude::LLVMTypeRef};
use std::{marker::PhantomData, pin::Pin};

pub(crate) enum LLType<'c> {
    I64(LLTypeRef<'c>),
    I32(LLTypeRef<'c>),
    F64(LLTypeRef<'c>),
    F32(LLTypeRef<'c>),
}

pub(crate) struct LLTypeRef<'c> {
    type_ref: LLVMTypeRef,
    _context: PhantomData<&'c ()>,
}

impl<'c> LLTypeRef<'c> {
    pub(crate) fn new(type_ref: LLVMTypeRef) -> Self {
        Self {
            type_ref,
            _context: PhantomData,
        }
    }
}

impl<'c> LLType<'c> {
    pub(crate) fn as_ptr(&self) -> LLVMTypeRef {
        match self {
            LLType::I64(t) => t.type_ref,
            LLType::I32(t) => t.type_ref,
            LLType::F64(t) => t.type_ref,
            LLType::F32(t) => t.type_ref,
        }
    }
}

/// LLFunctionType is a self-referential type.
pub(crate) struct LLFunctionType<'t> {
    pub(crate) params: Vec<LLVMTypeRef>,
    pub(crate) result: LLVMTypeRef,
    pub(crate) type_ref: LLVMTypeRef,
    _types: PhantomData<&'t ()>,
}

impl<'t> LLFunctionType<'t> {
    pub(crate) fn new(
        params: &'t [LLType],
        result: &'t LLType,
        is_varargs: bool,
    ) -> Pin<Box<Self>> {
        let mut this = Self {
            params: params.iter().map(|p| p.as_ptr()).collect::<Vec<_>>(),
            result: result.as_ptr(),
            type_ref: std::ptr::null_mut(),
            _types: PhantomData,
        };

        this.type_ref = unsafe {
            LLVMFunctionType(
                this.result,
                this.params.as_mut_ptr(),
                this.params.len() as u32,
                is_varargs as i32,
            )
        };

        Box::pin(this)
    }

    pub(crate) fn as_ptr(&self) -> LLVMTypeRef {
        self.type_ref
    }
}
