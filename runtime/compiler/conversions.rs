use llvm::{
    context::LLContext,
    types::{LLFunctionType, LLNumType, LLResultType},
};

use crate::types::{FuncType, NumType, RefType, ValType};

use super::{DataKind, ElementKind};

//------------------------------------------------------------------------------
// Wasmo <-> Wasmparser Conversions
//------------------------------------------------------------------------------

impl From<&wasmparser::Type> for ValType {
    fn from(value: &wasmparser::Type) -> Self {
        match value {
            wasmparser::Type::I32 => ValType::Num(NumType::I32),
            wasmparser::Type::I64 => ValType::Num(NumType::I64),
            wasmparser::Type::F32 => ValType::Num(NumType::F32),
            wasmparser::Type::F64 => ValType::Num(NumType::F64),
            wasmparser::Type::V128 => ValType::Vec,
            wasmparser::Type::FuncRef => ValType::Ref(RefType::FuncRef),
            wasmparser::Type::ExternRef => ValType::Ref(RefType::ExternRef),
        }
    }
}

impl From<&wasmparser::FuncType> for FuncType {
    fn from(value: &wasmparser::FuncType) -> Self {
        FuncType {
            params: value.params.iter().map(ValType::from).collect::<Vec<_>>(),
            results: value.returns.iter().map(ValType::from).collect::<Vec<_>>(),
        }
    }
}

impl<'a> From<&wasmparser::DataKind<'a>> for DataKind {
    fn from(value: &wasmparser::DataKind) -> Self {
        match value {
            wasmparser::DataKind::Passive => DataKind::Passive,
            wasmparser::DataKind::Active { memory_index, .. } => DataKind::Active {
                memory_index: *memory_index,
            },
        }
    }
}

impl<'a> From<&wasmparser::ElementKind<'a>> for ElementKind {
    fn from(value: &wasmparser::ElementKind) -> Self {
        match value {
            wasmparser::ElementKind::Passive => ElementKind::Passive,
            wasmparser::ElementKind::Declared => ElementKind::Declared,
            wasmparser::ElementKind::Active { table_index, .. } => ElementKind::Active {
                table_index: *table_index,
            },
        }
    }
}

//------------------------------------------------------------------------------
// LLVM <-> Wasmparser Conversions
//------------------------------------------------------------------------------

/// Converts `wasmparser` `Type` to `LLNumType`.
pub(crate) fn wasmparser_to_llvm_numtype(ctx: &LLContext, ty: &wasmparser::Type) -> LLNumType {
    match ty {
        wasmparser::Type::I32 => ctx.i32_type(),
        wasmparser::Type::I64 => ctx.i64_type(),
        wasmparser::Type::F32 => ctx.f32_type(),
        wasmparser::Type::F64 => ctx.f64_type(),
        wasmparser::Type::V128 => ctx.i128_type(),
        // TODO(appcypher): Use ctx.target_ptr_type() or sth similar.
        wasmparser::Type::FuncRef => ctx.i64_type(),
        wasmparser::Type::ExternRef => ctx.i64_type(),
    }
}

/// Converts `wasmparser` `FuncType` to `LLFunctionType`.
pub(crate) fn wasmparser_to_llvm_functype(
    ctx: &LLContext,
    ty: &wasmparser::FuncType,
) -> LLFunctionType {
    let params = ty
        .params
        .iter()
        .map(|i| wasmparser_to_llvm_numtype(ctx, i))
        .collect::<Vec<_>>();

    // If no result type, use a void.
    // If single result type, use a single valtype.
    // If multiple result types, use a tuple of valtypes.
    let result = match &ty.returns[..] {
        &[] => LLResultType::Void(ctx.void_type()),
        &[ref single_ty] => LLResultType::Num(wasmparser_to_llvm_numtype(ctx, single_ty)),
        result_types => {
            let types = result_types
                .iter()
                .map(|i| wasmparser_to_llvm_numtype(ctx, i))
                .collect::<Vec<_>>();

            LLResultType::Struct(ctx.struct_type(&types, true))
        }
    };

    ctx.function_type(&params, &result, false)
}

//------------------------------------------------------------------------------
// LLVM <-> Wasmo Conversions
//------------------------------------------------------------------------------

/// Converts `wasmparser` `Type` to `LLNumType`.
pub(crate) fn wasmo_to_llvm_numtype(ctx: &LLContext, ty: &ValType) -> LLNumType {
    use ValType::*;
    match ty {
        Num(NumType::I32) => ctx.i32_type(),
        Num(NumType::I64) => ctx.i64_type(),
        Num(NumType::F32) => ctx.f32_type(),
        Num(NumType::F64) => ctx.f64_type(),
        // TODO(appcypher): Use ctx.target_ptr_type()
        Ref(_) => ctx.i64_type(),
        Vec => ctx.i128_type(),
    }
}
