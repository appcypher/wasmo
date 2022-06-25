use llvm::{
    context::LLContext,
    types::{LLFunctionType, LLNumType, LLResultType},
};

use crate::types::{FuncType, NumType, RefType, ValType};

use super::{DataKind, ElementKind};

//------------------------------------------------------------------------------
// Wasmo <-> Wasmparser Conversions
//------------------------------------------------------------------------------

impl From<&wasmparser::ValType> for ValType {
    fn from(value: &wasmparser::ValType) -> Self {
        match value {
            wasmparser::ValType::I32 => ValType::Num(NumType::I32),
            wasmparser::ValType::I64 => ValType::Num(NumType::I64),
            wasmparser::ValType::F32 => ValType::Num(NumType::F32),
            wasmparser::ValType::F64 => ValType::Num(NumType::F64),
            wasmparser::ValType::V128 => ValType::Vec,
            wasmparser::ValType::FuncRef => ValType::Ref(RefType::FuncRef),
            wasmparser::ValType::ExternRef => ValType::Ref(RefType::ExternRef),
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
pub(crate) fn wasmparser_to_llvm_numtype(
    ctx: &LLContext,
    ty: &wasmparser::ValType,
) -> Box<dyn LLNumType> {
    match ty {
        wasmparser::ValType::I32 => Box::new(ctx.i32_type()),
        wasmparser::ValType::I64 => Box::new(ctx.i64_type()),
        wasmparser::ValType::F32 => Box::new(ctx.f32_type()),
        wasmparser::ValType::F64 => Box::new(ctx.f64_type()),
        wasmparser::ValType::V128 => Box::new(ctx.i128_type()),
        // TODO(appcypher): Use ctx.target_ptr_type() or sth similar.
        wasmparser::ValType::FuncRef => Box::new(ctx.i64_type()),
        wasmparser::ValType::ExternRef => Box::new(ctx.i64_type()),
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
    let result: Box<dyn LLResultType> = match &ty.returns[..] {
        &[] => Box::new(ctx.void_type()),
        &[ref single_ty] => {
            let num_type = wasmparser_to_llvm_numtype(ctx, single_ty);
            let result_type: &dyn LLResultType = num_type.as_ref().up();
            dyn_clone::clone_box(result_type)
        }
        result_types => {
            let types = result_types
                .iter()
                .map(|i| wasmparser_to_llvm_numtype(ctx, i))
                .collect::<Vec<_>>();

            Box::new(ctx.struct_type(&types, true))
        }
    };

    ctx.function_type(&params, result.as_ref(), false)
}

//------------------------------------------------------------------------------
// LLVM <-> Wasmo Conversions
//------------------------------------------------------------------------------

/// Converts `wasmo` `ValType` to `LLNumType`.
pub(crate) fn wasmo_to_llvm_numtype(ctx: &LLContext, ty: &ValType) -> Box<dyn LLNumType> {
    use ValType::*;
    match ty {
        Num(NumType::I32) => Box::new(ctx.i32_type()),
        Num(NumType::I64) => Box::new(ctx.i64_type()),
        Num(NumType::F32) => Box::new(ctx.f32_type()),
        Num(NumType::F64) => Box::new(ctx.f64_type()),
        // TODO(appcypher): Use ctx.target_ptr_type()
        Ref(_) => Box::new(ctx.i64_type()),
        Vec => Box::new(ctx.i128_type()),
    }
}
