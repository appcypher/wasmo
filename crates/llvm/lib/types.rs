use dyn_clone::DynClone;
use llvm_sys::{
    core::{
        LLVMConstInt, LLVMConstNull, LLVMConstReal, LLVMDoubleTypeInContext,
        LLVMFloatTypeInContext, LLVMFunctionType, LLVMGetTypeKind, LLVMInt128TypeInContext,
        LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMStructType, LLVMStructTypeInContext,
        LLVMVoidTypeInContext,
    },
    prelude::LLVMTypeRef,
    LLVMTypeKind,
};
use upcast::{Upcast, UpcastFrom};

use crate::{
    impl_trait, not_null,
    values::{LLConstFloat, LLConstInt, LLZero},
};

use super::context::LLContext;

//------------------------------------------------------------------------------
// Macros
//------------------------------------------------------------------------------

macro_rules! create_type_struct {
    ($ty:ident => $llvm_fn:ident, $doc_title:expr => $doc_ref:expr) => {
        #[doc = $doc_title]
        ///
        /// # Safety
        /// Only a kind of each `LLVMTypeRef` is ever created. They are singletons and are never freed.
        ///
        /// ### References
        /// - https://llvm.org/doxygen/classllvm_1_1Type.html#details
        #[doc = ""]
        #[doc = "# References"]
        #[doc = $doc_ref]
        #[derive(Debug, Clone)]
        pub struct $ty(LLVMTypeRef);

        impl $ty {
            pub(super) fn new(context: &LLContext) -> Self {
                let context_ref = unsafe { context.as_ptr() };
                Self(unsafe { $crate::not_null!($llvm_fn(context_ref)) })
            }

            /// Returns the underlying LLVMValueRef of this value.
            ///
            /// # Safety
            /// - Unsafe because it exposes a raw pointer gotten from LLVM ffi.
            #[allow(unused)]
            pub(crate) unsafe fn as_ptr(&self) -> LLVMTypeRef {
                self.0
            }
        }
    };
}

//------------------------------------------------------------------------------
// Type Definitions
//------------------------------------------------------------------------------

/// For types that are integers.
pub trait LLIntType: LLNumType + Upcast<dyn LLNumType> {
    /// Returns the underlying LLVMTypeRef of this value.
    ///
    /// # Safety
    /// - Unsafe because it exposes a raw pointer gotten from LLVM ffi.
    unsafe fn int_ref(&self) -> LLVMTypeRef;

    /// Creates a new LLVM const int instruction.
    fn constant(&self, value: u64, sign_extended: bool) -> LLConstInt {
        LLConstInt::from_ptr(unsafe { LLVMConstInt(self.int_ref(), value, sign_extended as i32) })
    }
}

/// For types that are floating points.
pub trait LLFloatType: LLNumType {
    /// Returns the underlying LLVMTypeRef of this value.
    ///
    /// # Safety
    /// - Unsafe because it exposes a raw pointer gotten from LLVM ffi.
    unsafe fn float_ref(&self) -> LLVMTypeRef;

    /// Creates a new LLVM const int instruction.
    fn constant(&self, value: f64) -> LLConstFloat {
        LLConstFloat::from_ptr(unsafe { LLVMConstReal(self.float_ref(), value) })
    }
}

/// For types that are numerical in nature, i.e. integer and floating-point types.
///
/// Upcast allows us to cast LLNumType to LLResultType.
pub trait LLNumType:
    LLValueType + LLResultType + Upcast<dyn LLResultType> + Upcast<dyn LLValueType>
{
    /// Returns the underlying LLVMTypeRef of this value.
    ///
    /// # Safety
    /// - Unsafe because it exposes a raw pointer gotten from LLVM ffi.
    unsafe fn num_ref(&self) -> LLVMTypeRef;

    fn zero(&self) -> LLZero {
        LLZero::from_ptr(unsafe { LLVMConstNull(self.num_ref()) })
    }
}

/// For types that can be returned as a result. This is based on WebAssembly's `Result` type.
///
/// That is number, void and struct types.
///
/// DynClone helps us clone a &dyn ResultType as Box<dyn ResultType>.
pub trait LLResultType: DynClone {
    /// Returns the underlying LLVMTypeRef of this value.
    ///
    /// # Safety
    /// - Unsafe because it exposes a raw pointer gotten from LLVM ffi.
    unsafe fn result_ref(&self) -> LLVMTypeRef;
}

/// For types that can be used as values. This is based on WebAssembly's `Value` type.
///
/// That is number and struct types.
///
/// DynClone helps us clone a &dyn ResultType as Box<dyn ResultType>.
pub trait LLValueType: DynClone {
    /// Returns the underlying LLVMTypeRef of this value.
    ///
    /// # Safety
    /// - Unsafe because it exposes a raw pointer gotten from LLVM ffi.
    unsafe fn value_ref(&self) -> LLVMTypeRef;
}

create_type_struct! {
    LLInt32Type => LLVMInt32TypeInContext,
    "Wrapper for LLVM i32 type" => "https://llvm.org/docs/LangRef.html#integer-type"
}

create_type_struct! {
    LLInt64Type => LLVMInt64TypeInContext,
    "Wrapper for LLVM i64 type" => "https://llvm.org/docs/LangRef.html#integer-type"
}

create_type_struct! {
    LLInt128Type =>  LLVMInt128TypeInContext,
    "Wrapper for LLVM i128 type" => "https://llvm.org/docs/LangRef.html#integer-type"
}

create_type_struct! {
    LLFloat32Type => LLVMFloatTypeInContext,
    "Wrapper for LLVM f32 type" => "https://llvm.org/docs/LangRef.html#floating-point-types"
}

create_type_struct! {
    LLFloat64Type => LLVMDoubleTypeInContext,
    "Wrapper for LLVM f64 type" => "https://llvm.org/docs/LangRef.html#floating-point-types"
}

create_type_struct! {
    LLVoidType => LLVMVoidTypeInContext,
    "Wrapper for LLVM void type" => "https://llvm.org/docs/LangRef.html#void-type"
}

/// Wrapper for LLVM pointer types (e.g. i64*, [2 x double]*).
///
/// # Safety
/// See [`LLNumType`](struct.LLNumType.html)
///
/// - https://llvm.org/docs/LangRef.html#pointer-type
pub struct LLPointerType(LLVMTypeRef);

/// Wrapper for LLVM vector types (e.g. <4 x i64>).
///
/// # Safety
/// See [`LLNumType`](struct.LLNumType.html)
///
/// # References
/// - https://llvm.org/docs/LangRef.html#vector-type
pub struct LLVectorType(LLVMTypeRef);

/// Wrapper for LLVM array type (e.g. [4 x double]).
///
/// # Safety
/// See [`LLNumType`](struct.LLNumType.html)
///
/// # References
/// - https://llvm.org/docs/LangRef.html#array-type
pub struct LLArrayType(LLVMTypeRef);

/// Wrapper for LLVM struct type.
///
/// # Safety
/// Same as [`LLFunctionType`](struct.LLFunctionType.html)
#[derive(Debug, Clone)]
pub struct LLStructType(LLVMTypeRef);

/// Wrapper for LLVM function type.
///
/// # Safety
/// Function types are a bit more complicated than scalar types because we need to allocate the array of types that gets passed to it.
///
/// The good thing however is that LLVM does not depend on our base pointer. They are reallocated by LLVM context.
///
/// ### References
/// - https://llvm.org/doxygen/Type_8cpp_source.html#l00361
///
/// # Ownership
/// - Owned by LLVM context.
#[derive(Debug)]
pub struct LLFunctionType(LLVMTypeRef);

//--------------------------------------------------------------------------------------------------
// Implementations
//--------------------------------------------------------------------------------------------------

impl LLStructType {
    /// Creates a new LLVM array type.
    ///
    /// # Safety
    /// See [LLStructType](struct.LLStructType.html) for safety.
    pub fn new(types: &[Box<dyn LLNumType>], is_packed: bool) -> Self {
        let types = types
            .iter()
            .map(|p| unsafe { p.num_ref() })
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
    ///
    /// # Safety
    /// See [LLFunctionType](struct.LLFunctionType.html) for safety.
    pub fn new(params: &[Box<dyn LLNumType>], result: &dyn LLResultType, is_varargs: bool) -> Self {
        let params = params
            .iter()
            .map(|p| unsafe { p.num_ref() })
            .collect::<Vec<_>>();

        Self(unsafe {
            not_null!(LLVMFunctionType(
                result.result_ref(),
                params.as_ptr() as *mut LLVMTypeRef,
                params.len() as u32,
                is_varargs as i32,
            ))
        })
    }

    pub(super) unsafe fn as_ptr(&self) -> LLVMTypeRef {
        self.0
    }
}

impl_trait! {
    LLIntType(int_ref -> LLVMTypeRef) for {
        LLInt32Type,
        LLInt64Type,
    }
}

impl_trait! {
    LLFloatType(float_ref -> LLVMTypeRef) for {
        LLFloat32Type,
        LLFloat64Type
    }
}

impl_trait! {
    LLNumType(num_ref -> LLVMTypeRef) for {
        LLInt32Type,
        LLInt64Type,
        LLInt128Type,
        LLFloat32Type,
        LLFloat64Type
    }
}

impl_trait! {
    LLValueType(value_ref -> LLVMTypeRef) for {
        LLInt32Type,
        LLInt64Type,
        LLInt128Type,
        LLFloat32Type,
        LLFloat64Type,
        LLStructType,
    }
}

impl_trait! {
    LLResultType(result_ref -> LLVMTypeRef) for {
        LLInt32Type,
        LLInt64Type,
        LLInt128Type,
        LLFloat32Type,
        LLFloat64Type,
        LLVoidType,
        LLStructType,
    }
}

/// The upcast library allows us to cast a trait to a supertrait.
impl<'a, T: LLResultType + 'a> UpcastFrom<T> for dyn LLResultType + 'a {
    fn up_from(value: &T) -> &(dyn LLResultType + 'a) {
        value
    }

    fn up_from_mut(value: &mut T) -> &mut (dyn LLResultType + 'a) {
        value
    }
}

impl<'a, T: LLValueType + 'a> UpcastFrom<T> for dyn LLValueType + 'a {
    fn up_from(value: &T) -> &(dyn LLValueType + 'a) {
        value
    }

    fn up_from_mut(value: &mut T) -> &mut (dyn LLValueType + 'a) {
        value
    }
}

impl<'a, T: LLNumType + 'a> UpcastFrom<T> for dyn LLNumType + 'a {
    fn up_from(value: &T) -> &(dyn LLNumType + 'a) {
        value
    }

    fn up_from_mut(value: &mut T) -> &mut (dyn LLNumType + 'a) {
        value
    }
}
