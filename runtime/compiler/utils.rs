pub mod convert {
    use anyhow::Result;
    use llvm::{
        context::LLContext,
        types::{LLFunctionType, LLNumType, LLResultType},
    };

    use crate::{
        compiler::{DataKind, ElementKind},
        errors::CompilerError,
        types::{FuncType, NumType, RefType, ValType},
    };

    /// Converts `wasmparser` `FuncType` to `wasmo` `FuncType`.
    pub fn to_wasmo_functype(ty: &wasmparser::FuncType) -> Result<FuncType> {
        let params = ty
            .params
            .iter()
            .map(to_wasmo_valtype)
            .collect::<Result<Vec<_>>>()?;

        let results = ty
            .returns
            .iter()
            .map(to_wasmo_valtype)
            .collect::<Result<Vec<_>>>()?;

        Ok(FuncType { params, results })
    }

    /// Converts `wasmparser` `ValType` to `wasmo` `ValType`.
    pub fn to_wasmo_valtype(ty: &wasmparser::Type) -> Result<ValType> {
        match ty {
            wasmparser::Type::I32 => Ok(ValType::Num(NumType::I32)),
            wasmparser::Type::I64 => Ok(ValType::Num(NumType::I64)),
            wasmparser::Type::F32 => Ok(ValType::Num(NumType::F32)),
            wasmparser::Type::F64 => Ok(ValType::Num(NumType::F64)),
            wasmparser::Type::V128 => Ok(ValType::Vec),
            wasmparser::Type::FuncRef => Ok(ValType::Ref(RefType::FuncRef)),
            wasmparser::Type::ExternRef => Ok(ValType::Ref(RefType::ExternRef)),
            t => Err(CompilerError::UnsupportedWasmoValType(format!("{:?}", t)).into()),
        }
    }

    /// Converts `wasmparser` `DataKind` to `wasmo` `DataKind`.
    pub fn to_wasmo_data_kind(ty: &wasmparser::DataKind) -> DataKind {
        match ty {
            wasmparser::DataKind::Passive => DataKind::Passive,
            wasmparser::DataKind::Active { memory_index, .. } => DataKind::Active {
                memory_index: *memory_index,
            },
        }
    }

    /// Converts `wasmparser` `ElementKind` to `wasmo` `ElementKind`.
    pub fn to_wasmo_element_kind(ty: &wasmparser::ElementKind) -> ElementKind {
        match ty {
            wasmparser::ElementKind::Passive => ElementKind::Passive,
            wasmparser::ElementKind::Declared => ElementKind::Declared,
            wasmparser::ElementKind::Active { table_index, .. } => ElementKind::Active {
                table_index: *table_index,
            },
        }
    }

    /// Converts `wasmo` `ValType` to `LLNumType`.
    pub(crate) fn to_llvm_valtype(ctx: &LLContext, ty: &ValType) -> LLNumType {
        use ValType::*;
        match ty {
            Num(NumType::I32) => ctx.i32_type(),
            Num(NumType::I64) => ctx.i64_type(),
            Num(NumType::F32) => ctx.f32_type(),
            Num(NumType::F64) => ctx.f64_type(),
            Ref(_) => ctx.i64_type(), // TODO(appcypher): Use ctx.target_ptr_type()
            Vec => ctx.i128_type(),
        }
    }

    /// Converts `wasmo` `ValType` to `LLFunctionType`.
    pub(crate) fn to_llvm_functype(ctx: &LLContext, ty: &FuncType) -> LLFunctionType {
        let params = ty
            .params
            .iter()
            .map(|i| to_llvm_valtype(ctx, i))
            .collect::<Vec<_>>();

        // If no result type, use a void.
        // If single result type, use a single valtype.
        // If multiple result types, use a tuple of valtypes.
        let result = match &ty.results[..] {
            &[] => LLResultType::Void(ctx.void_type()),
            &[ref single_ty] => LLResultType::Num(to_llvm_valtype(ctx, single_ty)),
            result_types => {
                let types = result_types
                    .iter()
                    .map(|i| to_llvm_valtype(ctx, i))
                    .collect::<Vec<_>>();

                LLResultType::Struct(ctx.struct_type(&types, true))
            }
        };

        ctx.function_type(&params, &result, false)
    }
}
