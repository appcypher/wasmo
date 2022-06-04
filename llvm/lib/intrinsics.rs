use llvm_sys::{
    core::{LLVMDoubleType, LLVMFloatType, LLVMFunctionType, LLVMInt32Type, LLVMInt64Type},
    prelude::LLVMTypeRef,
};

macro_rules! intrinsic {
    ($name:expr, ( $($params:ident),* ) -> $result:ident) => {
        Intrinsic {
            name: $name,
            get_type: || unsafe {
                let mut params = [$($params()),*];
                LLVMFunctionType($result(), params.as_mut_ptr(), params.len() as u32, 0)
            },
        }
    };
}

/// An LLVM intrinsic.
pub struct Intrinsic {
    pub(crate) name: &'static str,
    pub(crate) get_type: fn() -> LLVMTypeRef,
}

pub const FSHL_I32: Intrinsic =
    intrinsic!("llvm.fshl.i32", (LLVMInt32Type, LLVMInt32Type, LLVMInt32Type) -> LLVMInt32Type);

pub const FSHL_I64: Intrinsic =
    intrinsic!("llvm.fshl.i64", (LLVMInt64Type, LLVMInt64Type, LLVMInt64Type) -> LLVMInt64Type);

pub const FSHR_I32: Intrinsic =
    intrinsic!("llvm.fshr.i32", (LLVMInt32Type, LLVMInt32Type, LLVMInt32Type) -> LLVMInt32Type);

pub const FSHR_I64: Intrinsic =
    intrinsic!("llvm.fshr.i64", (LLVMInt64Type, LLVMInt64Type, LLVMInt64Type) -> LLVMInt64Type);

pub const CTLZ_I32: Intrinsic = intrinsic!("llvm.ctlz.i32", (LLVMInt32Type) -> LLVMInt32Type);

pub const CTLZ_I64: Intrinsic = intrinsic!("llvm.ctlz.i64", (LLVMInt64Type) -> LLVMInt64Type);

pub const CTTZ_I32: Intrinsic = intrinsic!("llvm.cttz.i32", (LLVMInt32Type) -> LLVMInt32Type);

pub const CTTZ_I64: Intrinsic = intrinsic!("llvm.cttz.i64", (LLVMInt64Type) -> LLVMInt64Type);

pub const CTPOP_I32: Intrinsic = intrinsic!("llvm.ctpop.i32", (LLVMInt32Type) -> LLVMInt32Type);

pub const CTPOP_I64: Intrinsic = intrinsic!("llvm.ctpop.i64", (LLVMInt64Type) -> LLVMInt64Type);

pub const MINIMUM_F32: Intrinsic =
    intrinsic!("llvm.minimum.f32", (LLVMFloatType, LLVMFloatType) -> LLVMFloatType);

pub const MINIMUM_F64: Intrinsic =
    intrinsic!("llvm.minimum.f64", (LLVMDoubleType, LLVMDoubleType) -> LLVMDoubleType);

pub const MAXIMUM_F32: Intrinsic =
    intrinsic!("llvm.maximum.f32", (LLVMFloatType, LLVMFloatType) -> LLVMFloatType);

pub const MAXIMUM_F64: Intrinsic =
    intrinsic!("llvm.maximum.f64", (LLVMDoubleType, LLVMDoubleType) -> LLVMDoubleType);

pub const COPYSIGN_F32: Intrinsic =
    intrinsic!("llvm.copysign.f32", (LLVMFloatType, LLVMFloatType) -> LLVMFloatType);

pub const COPYSIGN_F64: Intrinsic =
    intrinsic!("llvm.copysign.f64", (LLVMDoubleType, LLVMDoubleType) -> LLVMDoubleType);

pub const ABS_F32: Intrinsic = intrinsic!("llvm.fabs.f32", (LLVMFloatType) -> LLVMFloatType);

pub const ABS_F64: Intrinsic = intrinsic!("llvm.fabs.f64", (LLVMDoubleType) -> LLVMDoubleType);

pub const NEG_F32: Intrinsic = intrinsic!("llvm.fneg.f32", (LLVMFloatType) -> LLVMFloatType);

pub const NEG_F64: Intrinsic = intrinsic!("llvm.fneg.f64", (LLVMDoubleType) -> LLVMDoubleType);

pub const SQRT_F32: Intrinsic = intrinsic!("llvm.sqrt.f32", (LLVMFloatType) -> LLVMFloatType);

pub const SQRT_F64: Intrinsic = intrinsic!("llvm.sqrt.f64", (LLVMDoubleType) -> LLVMDoubleType);

pub const CEIL_F32: Intrinsic = intrinsic!("llvm.ceil.f32", (LLVMFloatType) -> LLVMFloatType);

pub const CEIL_F64: Intrinsic = intrinsic!("llvm.ceil.f64", (LLVMDoubleType) -> LLVMDoubleType);

pub const FLOOR_F32: Intrinsic = intrinsic!("llvm.floor.f32", (LLVMFloatType) -> LLVMFloatType);

pub const FLOOR_F64: Intrinsic = intrinsic!("llvm.floor.f64", (LLVMDoubleType) -> LLVMDoubleType);

pub const TRUNC_F32: Intrinsic = intrinsic!("llvm.trunc.f32", (LLVMFloatType) -> LLVMFloatType);

pub const TRUNC_F64: Intrinsic = intrinsic!("llvm.trunc.f64", (LLVMDoubleType) -> LLVMDoubleType);

pub const ROUND_EVEN_F32: Intrinsic =
    intrinsic!("llvm.trunc.f32", (LLVMFloatType) -> LLVMFloatType);

pub const ROUND_EVEN_F64: Intrinsic =
    intrinsic!("llvm.trunc.f64", (LLVMDoubleType) -> LLVMDoubleType);
