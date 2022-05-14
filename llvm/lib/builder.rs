use std::ffi::CString;

use anyhow::Result;
use llvm_sys::{
    core::{
        LLVMBuildAlloca, LLVMBuildBr, LLVMBuildCondBr, LLVMBuildRet, LLVMBuildRetVoid,
        LLVMBuildStore, LLVMBuildUnreachable, LLVMCreateBuilderInContext, LLVMDisposeBuilder,
        LLVMPositionBuilderAtEnd,
    },
    prelude::LLVMBuilderRef,
};

use crate::{
    basic_block::LLBasicBlock,
    context::LLContext,
    types::LLNumType,
    values::{LLAlloca, LLBr, LLCondBr, LLRet, LLRetVoid, LLStore, LLUnreachable, LLValue},
};

/// LLVM Builder wrapper.
///
/// # Ownership
/// - Not owned by anything.
pub struct LLBuilder(LLVMBuilderRef);

impl LLBuilder {
    /// Creates a new LLVM IRBuilder.
    pub fn new(context: &LLContext) -> Self {
        Self(unsafe { LLVMCreateBuilderInContext(context.as_ptr()) })
    }

    /// Puts the builder at the end of the given basic block.
    pub fn position_at_end(&self, basic_block: &LLBasicBlock) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.0, basic_block.as_ptr());
        }
    }

    /// Creates a new LLVM Alloca instruction.
    pub fn build_alloca(&mut self, ty: &LLNumType, name: &str) -> Result<LLAlloca> {
        Ok(LLAlloca::from_ptr(unsafe {
            LLVMBuildAlloca(self.0, ty.as_ptr(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM Store instruction.
    pub fn build_store(&mut self, alloca: &LLAlloca, value: &impl LLValue) -> LLStore {
        LLStore::from_ptr(unsafe { LLVMBuildStore(self.0, alloca.as_ptr(), value.value_ref()) })
    }

    /// Creates a new LLVM Unreachable instruction.
    pub fn build_unreachable(&mut self) -> LLUnreachable {
        LLUnreachable::from_ptr(unsafe { LLVMBuildUnreachable(self.0) })
    }

    /// Creates a new LLVM Ret instruction.
    pub fn build_ret(&mut self, value: &impl LLValue) -> LLRet {
        LLRet::from_ptr(unsafe { LLVMBuildRet(self.0, value.value_ref()) })
    }

    /// Creates a new LLVM Ret Void instruction.
    pub fn build_ret_void(&mut self) -> LLRetVoid {
        LLRetVoid::from_ptr(unsafe { LLVMBuildRetVoid(self.0) })
    }

    /// Creates a new LLVM Br instruction.
    pub fn build_br(&mut self, basic_block: &LLBasicBlock) -> LLBr {
        LLBr::from_ptr(unsafe { LLVMBuildBr(self.0, basic_block.as_ptr()) })
    }

    /// Creates a new LLVM BrIf instruction.
    pub fn build_cond_br(
        &mut self,
        cond: &dyn LLValue,
        then_block: &LLBasicBlock,
        else_block: &LLBasicBlock,
    ) -> LLCondBr {
        LLCondBr::from_ptr(unsafe {
            LLVMBuildCondBr(
                self.0,
                cond.value_ref(),
                then_block.as_ptr(),
                else_block.as_ptr(),
            )
        })
    }

    pub(crate) unsafe fn as_ptr(&self) -> LLVMBuilderRef {
        self.0
    }
}

impl Drop for LLBuilder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.0);
        }
    }
}
