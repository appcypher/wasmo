use anyhow::Result;
use dyn_clone::DynClone;
use std::ffi::CString;

use llvm_sys::{
    core::{LLVMAddFunction, LLVMGetParam, LLVMGetTypeKind, LLVMTypeOf},
    prelude::LLVMValueRef,
    LLVMIntPredicate, LLVMRealPredicate, LLVMTypeKind,
};

use crate::{basic_block::LLBasicBlock, context::LLContext, impl_trait, not_null};

use super::{module::LLModule, types::LLFunctionType};

//-----------------------------------------------------------------------------
// Macros
//-----------------------------------------------------------------------------

// TODO(appcypher): Support adding link to instruction like create_type_struct macro.
macro_rules! create_value_struct {
    ($ty:ident, $doc_title:expr) => {
        #[doc = $doc_title]
        /// # Ownership
        /// - Owned by an LLVM Module.
        ///
        /// https://llvm.org/doxygen/classllvm_1_1Value.html#details
        // TODO(appcyher): Cloning is not great. Possible Rc in the future.
        #[derive(Debug, Clone)]
        pub struct $ty(LLVMValueRef);

        impl $ty {
            pub(super) fn from_ptr(ptr: LLVMValueRef) -> Self {
                Self($crate::not_null!(ptr))
            }

            /// Returns the underlying LLVMValueRef of this value.
            ///
            /// # Safety
            /// - Unsafe because it exposes a raw pointer gotten from LLVM ffi.
            #[allow(unused)]
            pub(crate) unsafe fn as_ptr(&self) -> LLVMValueRef {
                self.0
            }
        }
    };
}

//-----------------------------------------------------------------------------
// Traits
//-----------------------------------------------------------------------------

/// For types that are LLVMValueRef.
pub trait LLValue: DynClone {
    /// Returns the underlying LLVMValueRef of this value.
    ///
    /// # Safety
    /// - Unsafe because it exposes a raw pointer gotten from LLVM ffi.
    unsafe fn value_ref(&self) -> LLVMValueRef;

    fn is_pointer_type(&self) -> bool {
        unsafe {
            LLVMTypeKind::LLVMPointerTypeKind == LLVMGetTypeKind(LLVMTypeOf(self.value_ref()))
        }
    }
}

//------------------------------------------------------------------------------
// Type Definitions
//------------------------------------------------------------------------------

/// LLVM Function wrapper.
///
/// # Safety
/// It is unsafe to use the reference of `LLFunctionType` because its params can be independently freed.
/// Holding an `Rc` to it ensures that that does not happen.
///
/// WARNING: This is safe only if we can only create a Function from a Module.
///
/// # Ownership
/// - Owned by an LLVM Module.
/// - Owns the basic blocks and arguments added to it.
///
/// - https://llvm.org/doxygen/Function_8cpp_source.html#l00409
/// - https://llvm.org/doxygen/Function_8cpp_source.html#l00509
/// - https://llvm.org/doxygen/classllvm_1_1Value.html#details
#[derive(Debug, Clone)]
pub struct LLFunction(LLVMValueRef);

#[derive(Debug, Clone, Copy)]
pub enum LLIntPredicate {
    EQ,
    NE,
    UGT,
    UGE,
    ULT,
    ULE,
    SGT,
    SGE,
    SLT,
    SLE,
}

#[derive(Debug, Clone, Copy)]
pub enum LLFloatPredicate {
    OEQ,
    ONE,
    OGT,
    OGE,
    OLT,
    OLE,
    UEQ,
    UNE,
    UGT,
    UGE,
    ULT,
    ULE,
}

create_value_struct! {
    LLParam,
    "Wraps the LLVM param value"
}

create_value_struct! {
    LLUnreachable,
    "Wraps the LLVM unreachable value"
}

create_value_struct! {
    LLAlloca,
    "Wraps the LLVM alloca value"
}

create_value_struct! {
    LLStore,
    "Wraps the LLVM store value"
}

create_value_struct! {
    LLLoad,
    "Wraps the LLVM load value"
}

create_value_struct! {
    LLGEP,
    "Wraps the LLVM getelementptr value"
}

create_value_struct! {
    LLRet,
    "Wraps the LLVM ret value"
}

create_value_struct! {
    LLRetVoid,
    "Wraps the LLVM ret void value"
}

create_value_struct! {
    LLBr,
    "Wraps the LLVM br value"
}

create_value_struct! {
    LLCondBr,
    "Wraps the LLVM cond br value"
}

create_value_struct! {
    LLCall,
    "Wraps the LLVM call value"
}

create_value_struct! {
    LLIntAdd,
    "Wraps the LLVM add value"
}

create_value_struct! {
    LLIntSub,
    "Wraps the LLVM sub value"
}

create_value_struct! {
    LLIntMul,
    "Wraps the LLVM mul value"
}

create_value_struct! {
    LLIntUDiv,
    "Wraps the LLVM udiv value"
}

create_value_struct! {
    LLIntSDiv,
    "Wraps the LLVM sdiv value"
}

create_value_struct! {
    LLIntURem,
    "Wraps the LLVM urem value"
}

create_value_struct! {
    LLIntSRem,
    "Wraps the LLVM srem value"
}
create_value_struct! {
    LLIntAnd,
    "Wraps the LLVM and value"
}

create_value_struct! {
    LLIntOr,
    "Wraps the LLVM or value"
}

create_value_struct! {
    LLIntXor,
    "Wraps the LLVM xor value"
}

create_value_struct! {
    LLIntShl,
    "Wraps the LLVM shl value"
}

create_value_struct! {
    LLIntLShr,
    "Wraps the LLVM lshr value"
}

create_value_struct! {
    LLIntAShr,
    "Wraps the LLVM ashr value"
}

create_value_struct! {
    LLIntCmp,
    "Wraps the LLVM icmp [eq|ne|sge|...] value"
}

create_value_struct! {
    LLFloatAdd,
    "Wraps the LLVM fadd value"
}

create_value_struct! {
    LLFloatSub,
    "Wraps the LLVM fsub value"
}

create_value_struct! {
    LLFloatMul,
    "Wraps the LLVM fmul value"
}

create_value_struct! {
    LLFloatDiv,
    "Wraps the LLVM fdiv value"
}

create_value_struct! {
    LLFloatRem,
    "Wraps the LLVM frem value"
}

create_value_struct! {
    LLZero,
    "Wraps the LLVM zero value"
}

create_value_struct! {
    LLFloatCmp,
    "Wraps the LLVM fcmp [ueq|une|uge|...] value"
}

create_value_struct! {
    LLConstStruct,
    "Wraps the LLVM const struct value"
}

create_value_struct! {
    LLConstInt,
    "Wraps the LLVM const int value"
}

create_value_struct! {
    LLConstFloat,
    "Wraps the LLVM const float value"
}

//------------------------------------------------------------------------------
// Implementations
//------------------------------------------------------------------------------

impl LLFunction {
    /// Creates a new LLVM function.
    ///
    /// This is the only way to create an LLFunction, ensuring it has an associated Module.
    /// Therefore a Module must already exist to dispose it.
    ///
    /// # Safety
    /// - Module can only be created from a Context that frees it.
    ///
    /// ### References
    /// - https://llvm.org/doxygen/Twine_8h_source.html#l00271
    /// - https://llvm.org/doxygen/Twine_8h_source.html#l00477
    /// - https://llvm.org/doxygen/Value_8cpp_source.html#l00315
    /// - https://llvm.org/doxygen/StringRef_8h_source.html#l00107
    pub(super) fn new(
        name: &str,
        module: &mut LLModule,
        function_type: &LLFunctionType,
    ) -> Result<Self> {
        Ok(Self(unsafe {
            not_null!(LLVMAddFunction(
                module.as_ptr(),
                CString::new(name)?.as_ptr(),
                function_type.as_ptr(),
            ))
        }))
    }

    pub(crate) fn from_ptr(ptr: LLVMValueRef) -> Self {
        Self(ptr)
    }

    /// Creates a new LLVM basic block.
    pub fn create_and_append_basic_block(
        &self,
        name: &str,
        context: &LLContext,
    ) -> Result<LLBasicBlock> {
        LLBasicBlock::create_and_append(name, self, context)
    }

    // Appends a basic block to the end of the function.
    pub fn append_basic_block(&self, basic_block: &mut LLBasicBlock) {
        basic_block.append_to_function(self);
    }

    /// Gets the param at the given index.
    pub fn get_param(&self, index: u32) -> LLParam {
        LLParam::from_ptr(unsafe { LLVMGetParam(self.as_ptr(), index) })
    }

    pub(crate) unsafe fn as_ptr(&self) -> LLVMValueRef {
        self.0
    }
}

impl_trait! {
    LLValue(value_ref -> LLVMValueRef) for {
        LLFunction,
        LLParam,
        LLUnreachable,
        LLAlloca,
        LLStore,
        LLLoad,
        LLGEP,
        LLRet,
        LLBr,
        LLCall,
        LLCondBr,
        LLIntAdd,
        LLIntSub,
        LLIntMul,
        LLIntUDiv,
        LLIntSDiv,
        LLIntURem,
        LLIntSRem,
        LLIntAnd,
        LLIntOr,
        LLIntXor,
        LLIntShl,
        LLIntLShr,
        LLIntAShr,
        LLIntCmp,
        LLFloatAdd,
        LLFloatSub,
        LLFloatMul,
        LLFloatDiv,
        LLFloatRem,
        LLFloatCmp,
        LLZero,
        LLRetVoid,
        LLConstStruct,
        LLConstInt,
        LLConstFloat,
    }
}

impl From<LLIntPredicate> for LLVMIntPredicate {
    fn from(pred: LLIntPredicate) -> Self {
        match pred {
            LLIntPredicate::EQ => LLVMIntPredicate::LLVMIntEQ,
            LLIntPredicate::NE => LLVMIntPredicate::LLVMIntNE,
            LLIntPredicate::UGT => LLVMIntPredicate::LLVMIntUGT,
            LLIntPredicate::UGE => LLVMIntPredicate::LLVMIntUGE,
            LLIntPredicate::ULT => LLVMIntPredicate::LLVMIntULT,
            LLIntPredicate::ULE => LLVMIntPredicate::LLVMIntULE,
            LLIntPredicate::SGT => LLVMIntPredicate::LLVMIntSGT,
            LLIntPredicate::SGE => LLVMIntPredicate::LLVMIntSGE,
            LLIntPredicate::SLT => LLVMIntPredicate::LLVMIntSLT,
            LLIntPredicate::SLE => LLVMIntPredicate::LLVMIntSLE,
        }
    }
}

impl From<LLFloatPredicate> for LLVMRealPredicate {
    fn from(pred: LLFloatPredicate) -> Self {
        match pred {
            LLFloatPredicate::OEQ => LLVMRealPredicate::LLVMRealOEQ,
            LLFloatPredicate::ONE => LLVMRealPredicate::LLVMRealONE,
            LLFloatPredicate::OGT => LLVMRealPredicate::LLVMRealOGT,
            LLFloatPredicate::OGE => LLVMRealPredicate::LLVMRealOGE,
            LLFloatPredicate::OLT => LLVMRealPredicate::LLVMRealOLT,
            LLFloatPredicate::OLE => LLVMRealPredicate::LLVMRealOLE,
            LLFloatPredicate::UEQ => LLVMRealPredicate::LLVMRealUEQ,
            LLFloatPredicate::UNE => LLVMRealPredicate::LLVMRealUNE,
            LLFloatPredicate::UGT => LLVMRealPredicate::LLVMRealUGT,
            LLFloatPredicate::UGE => LLVMRealPredicate::LLVMRealUGE,
            LLFloatPredicate::ULT => LLVMRealPredicate::LLVMRealULT,
            LLFloatPredicate::ULE => LLVMRealPredicate::LLVMRealULE,
        }
    }
}
