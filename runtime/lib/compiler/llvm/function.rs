// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use std::marker::PhantomData;
use utilities::result::Result;

use llvm_sys::{core::LLVMAddFunction, prelude::LLVMValueRef};

use super::{module::LLModule, types::LLFunctionType, utils::LLString};

pub(crate) struct LLFunction<'m, 's, 't> {
    function_ref: LLVMValueRef,
    _module: PhantomData<&'m ()>,
    _name: PhantomData<&'s ()>,
    _signature: PhantomData<&'t ()>,
}

impl<'m, 's, 't> LLFunction<'m, 's, 't> {
    pub(crate) fn attach(
        module: &'m LLModule,
        name: &'s LLString,
        signature: &'t LLFunctionType,
    ) -> Result<Self> {
        Ok(Self {
            function_ref: unsafe {
                LLVMAddFunction(module.as_ptr(), name.as_ptr(), signature.as_ptr())
            },
            _module: PhantomData,
            _name: PhantomData,
            _signature: PhantomData,
        })
    }
}
