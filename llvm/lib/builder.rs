use std::ffi::CString;

use anyhow::Result;
use llvm_sys::{
    core::{
        LLVMBuildAShr, LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildAnd, LLVMBuildBr, LLVMBuildCall, LLVMBuildCondBr,
        LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFMul, LLVMBuildFRem, LLVMBuildGEP, LLVMBuildICmp,
        LLVMBuildLShr, LLVMBuildLoad, LLVMBuildMul, LLVMBuildOr, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildSDiv,
        LLVMBuildSRem, LLVMBuildShl, LLVMBuildStore, LLVMBuildSub, LLVMBuildUDiv, LLVMBuildURem, LLVMBuildUnreachable,
        LLVMBuildXor, LLVMConstStruct, LLVMCreateBuilderInContext, LLVMDisposeBuilder, LLVMPositionBuilderAtEnd,
    },
    prelude::LLVMBuilderRef,
};

use crate::{
    basic_block::LLBasicBlock,
    context::LLContext,
    intrinsics::Intrinsic,
    module::LLModule,
    not_null,
    types::LLValueType,
    values::{
        LLAlloca, LLBr, LLCall, LLCondBr, LLConstStruct, LLFloatAdd, LLFloatCmp, LLFloatDiv, LLFloatMul,
        LLFloatPredicate, LLFloatRem, LLFloatSub, LLFunction, LLIntAShr, LLIntAdd, LLIntCmp, LLIntLShr, LLIntMul,
        LLIntOr, LLIntPredicate, LLIntSDiv, LLIntSRem, LLIntShl, LLIntSub, LLIntUDiv, LLIntURem, LLIntXor, LLLoad,
        LLRet, LLRetVoid, LLStore, LLUnreachable, LLValue, LLGEP,
    },
};

/// LLVM Builder wrapper.
///
/// # Ownership
/// - Not owned by anything.
pub struct LLBuilder(LLVMBuilderRef);

impl LLBuilder {
    /// Creates a new LLVM IRBuilder.
    pub(crate) fn new(context: &LLContext) -> Self {
        Self(unsafe { not_null!(LLVMCreateBuilderInContext(context.as_ptr())) })
    }

    /// Puts the builder at the end of the given basic block.
    pub fn position_at_end(&self, basic_block: &LLBasicBlock) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.0, basic_block.as_ptr());
        }
    }

    /// Creates a new LLVM alloca instruction.
    pub fn build_alloca(&mut self, ty: &dyn LLValueType, name: &str) -> Result<LLAlloca> {
        Ok(LLAlloca::from_ptr(unsafe {
            LLVMBuildAlloca(self.0, ty.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM store instruction.
    pub fn build_store(&mut self, value: &dyn LLValue, alloca: &dyn LLValue) -> LLStore {
        LLStore::from_ptr(unsafe { LLVMBuildStore(self.0, value.value_ref(), alloca.value_ref()) })
    }

    /// Creates a new LLVM load instruction.
    pub fn build_load(&mut self, ptr: &dyn LLValue, name: &str) -> Result<LLLoad> {
        Ok(LLLoad::from_ptr(unsafe {
            LLVMBuildLoad(self.0, ptr.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    pub fn build_gep(&mut self, ptr: &dyn LLValue, indices: &[Box<dyn LLValue>], name: &str) -> Result<LLGEP> {
        Ok(LLGEP::from_ptr(unsafe {
            LLVMBuildGEP(
                self.0,
                ptr.value_ref(),
                indices.iter().map(|v| v.value_ref()).collect::<Vec<_>>().as_mut_ptr(),
                indices.len() as u32,
                CString::new(name)?.as_ptr(),
            )
        }))
    }

    /// Creates a new LLVM unreachable instruction.
    pub fn build_unreachable(&mut self) -> LLUnreachable {
        LLUnreachable::from_ptr(unsafe { LLVMBuildUnreachable(self.0) })
    }

    /// Creates a new LLVM ret instruction.
    pub fn build_ret(&mut self, value: &dyn LLValue) -> LLRet {
        LLRet::from_ptr(unsafe { LLVMBuildRet(self.0, value.value_ref()) })
    }

    /// Creates a new LLVM ret void instruction.
    pub fn build_ret_void(&mut self) -> LLRetVoid {
        LLRetVoid::from_ptr(unsafe { LLVMBuildRetVoid(self.0) })
    }

    /// Creates a new LLVM br instruction.
    pub fn build_br(&mut self, basic_block: &LLBasicBlock) -> LLBr {
        LLBr::from_ptr(unsafe { LLVMBuildBr(self.0, basic_block.as_ptr()) })
    }

    /// Creates a new LLVM call instruction.
    pub fn build_call(&mut self, func: &LLFunction, args: &[&dyn LLValue], name: &str) -> Result<LLCall> {
        Ok(LLCall::from_ptr(unsafe {
            LLVMBuildCall(
                self.0,
                func.value_ref(),
                args.iter().map(|v| v.value_ref()).collect::<Vec<_>>().as_mut_ptr(),
                args.len() as u32,
                CString::new(name)?.as_ptr(),
            )
        }))
    }

    /// Creates a new LLVM call instruction.
    pub fn build_call_intrinsic(
        &mut self,
        intrinsic: &Intrinsic,
        args: &[&dyn LLValue],
        module: &mut LLModule,
        name: &str,
    ) -> Result<LLCall> {
        let function = module.add_or_get_intrinsic_function(intrinsic)?;
        Ok(LLCall::from_ptr(unsafe {
            LLVMBuildCall(
                self.0,
                function.as_ptr(),
                args.iter().map(|v| v.value_ref()).collect::<Vec<_>>().as_mut_ptr(),
                args.len() as u32,
                CString::new(name)?.as_ptr(),
            )
        }))
    }

    /// Creates a new LLVM add instruction. Takes integer scalar and vector types.
    pub fn build_int_add(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLIntAdd> {
        Ok(LLIntAdd::from_ptr(unsafe {
            LLVMBuildAdd(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM sub instruction. Takes integer scalar and vector types.
    pub fn build_int_sub(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLIntSub> {
        Ok(LLIntSub::from_ptr(unsafe {
            LLVMBuildSub(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM mul instruction. Takes integer scalar and vector types.
    pub fn build_int_mul(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLIntMul> {
        Ok(LLIntMul::from_ptr(unsafe {
            LLVMBuildMul(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM udiv instruction. Takes integer scalar and vector types.
    pub fn build_int_udiv(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLIntUDiv> {
        Ok(LLIntUDiv::from_ptr(unsafe {
            LLVMBuildUDiv(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM sdiv instruction. Takes integer scalar and vector types.
    pub fn build_int_sdiv(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLIntSDiv> {
        Ok(LLIntSDiv::from_ptr(unsafe {
            LLVMBuildSDiv(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM urem instruction. Takes integer scalar and vector types.
    pub fn build_int_urem(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLIntURem> {
        Ok(LLIntURem::from_ptr(unsafe {
            LLVMBuildURem(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM srem instruction. Takes integer scalar and vector types.
    pub fn build_int_srem(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLIntSRem> {
        Ok(LLIntSRem::from_ptr(unsafe {
            LLVMBuildSRem(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM and instruction. Takes integer scalar and vector types.
    pub fn build_int_and(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLIntAdd> {
        Ok(LLIntAdd::from_ptr(unsafe {
            LLVMBuildAnd(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM or instruction. Takes integer scalar and vector types.
    pub fn build_int_or(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLIntOr> {
        Ok(LLIntOr::from_ptr(unsafe {
            LLVMBuildOr(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM xor instruction. Takes integer scalar and vector types.
    pub fn build_int_xor(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLIntXor> {
        Ok(LLIntXor::from_ptr(unsafe {
            LLVMBuildXor(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM shl instruction. Takes integer scalar and vector types.
    ///
    /// Can return poison value.
    pub fn build_int_shl(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLIntShl> {
        Ok(LLIntShl::from_ptr(unsafe {
            LLVMBuildShl(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM lshr instruction. Takes integer scalar and vector types.
    ///
    /// Can return poison value.
    pub fn build_int_lshr(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLIntLShr> {
        Ok(LLIntLShr::from_ptr(unsafe {
            LLVMBuildLShr(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM ashr instruction. Takes integer scalar and vector types.
    pub fn build_int_ashr(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLIntAShr> {
        Ok(LLIntAShr::from_ptr(unsafe {
            LLVMBuildAShr(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM icmp instruction. Takes integer scalar and vector types.
    pub fn build_int_cmp(
        &mut self,
        kind: LLIntPredicate,
        lhs: &dyn LLValue,
        rhs: &dyn LLValue,
        name: &str,
    ) -> Result<LLIntCmp> {
        Ok(LLIntCmp::from_ptr(unsafe {
            LLVMBuildICmp(
                self.0,
                kind.into(),
                lhs.value_ref(),
                rhs.value_ref(),
                CString::new(name)?.as_ptr(),
            )
        }))
    }

    /// Creates a new LLVM fcmp instruction. Takes floating-point scalar and vector types.
    pub fn build_float_cmp(
        &mut self,
        kind: LLFloatPredicate,
        lhs: &dyn LLValue,
        rhs: &dyn LLValue,
        name: &str,
    ) -> Result<LLFloatCmp> {
        Ok(LLFloatCmp::from_ptr(unsafe {
            LLVMBuildFCmp(
                self.0,
                kind.into(),
                lhs.value_ref(),
                rhs.value_ref(),
                CString::new(name)?.as_ptr(),
            )
        }))
    }

    /// Creates a new LLVM fadd instruction. Takes floating-point scalar and vector types.
    pub fn build_float_add(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLFloatAdd> {
        Ok(LLFloatAdd::from_ptr(unsafe {
            LLVMBuildFAdd(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM fsub instruction. Takes floating-point scalar and vector types.
    pub fn build_float_sub(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLFloatSub> {
        Ok(LLFloatSub::from_ptr(unsafe {
            LLVMBuildSub(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM fmul instruction. Takes floating-point scalar and vector types.
    pub fn build_float_mul(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLFloatMul> {
        Ok(LLFloatMul::from_ptr(unsafe {
            LLVMBuildFMul(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM fdiv instruction. Takes floating-point scalar and vector types.
    pub fn build_float_div(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLFloatDiv> {
        Ok(LLFloatDiv::from_ptr(unsafe {
            LLVMBuildFDiv(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM frem instruction. Takes floating-point scalar and vector types.
    pub fn build_float_rem(&mut self, lhs: &dyn LLValue, rhs: &dyn LLValue, name: &str) -> Result<LLFloatRem> {
        Ok(LLFloatRem::from_ptr(unsafe {
            LLVMBuildFRem(self.0, lhs.value_ref(), rhs.value_ref(), CString::new(name)?.as_ptr())
        }))
    }

    /// Creates a new LLVM brif instruction.
    pub fn build_cond_br(
        &mut self,
        cond: &dyn LLValue,
        then_block: &LLBasicBlock,
        else_block: &LLBasicBlock,
    ) -> LLCondBr {
        LLCondBr::from_ptr(unsafe {
            LLVMBuildCondBr(self.0, cond.value_ref(), then_block.as_ptr(), else_block.as_ptr())
        })
    }

    /// Creates a new struct value.
    pub fn build_struct(&mut self, values: &[Box<dyn LLValue>], packed: bool) -> LLConstStruct {
        LLConstStruct::from_ptr(unsafe {
            LLVMConstStruct(
                values.iter().map(|v| v.value_ref()).collect::<Vec<_>>().as_mut_ptr(),
                values.len() as u32,
                packed as i32,
            )
        })
    }

    #[allow(unused)]
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
