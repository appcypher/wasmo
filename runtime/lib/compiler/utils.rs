// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

pub mod convert {
    use crate::{
        compiler::{DataKind, ElementKind},
        errors::CompilerError,
        types::{FuncType, NumType, RefType, ValType},
    };
    use anyhow::Result;

    /// Converts `wasmparser` `FuncType` to `wasmo` `FuncType`.
    pub fn to_wasmo_functype(ty: &wasmparser::FuncType) -> Result<FuncType> {
        let params = ty
            .params
            .iter()
            .map(|i| to_wasmo_valtype(i))
            .collect::<Result<Vec<_>>>()?;

        let returns = ty
            .returns
            .iter()
            .map(|i| to_wasmo_valtype(i))
            .collect::<Result<Vec<_>>>()?;

        Ok(FuncType { params, returns })
    }

    /// Converts `wasmparser` `ValType` to `wasmo` `ValType`.
    pub fn to_wasmo_valtype(ty: &wasmparser::Type) -> Result<ValType> {
        match ty {
            wasmparser::Type::I32 => Ok(ValType::NumType(NumType::I32)),
            wasmparser::Type::I64 => Ok(ValType::NumType(NumType::I64)),
            wasmparser::Type::F32 => Ok(ValType::NumType(NumType::F32)),
            wasmparser::Type::F64 => Ok(ValType::NumType(NumType::F64)),
            wasmparser::Type::V128 => Ok(ValType::VecType),
            wasmparser::Type::FuncRef => Ok(ValType::RefType(RefType::FuncRef)),
            wasmparser::Type::ExternRef => Ok(ValType::RefType(RefType::ExternRef)),
            t => Err(CompilerError::UnsupportedValType(format!("{:?}", t)).into()),
        }
    }

    /// Converts `wasmo` `DataKind` to `wasmparser` `DataKind`.
    pub fn to_wasmo_data_kind(ty: &wasmparser::DataKind) -> DataKind {
        match ty {
            wasmparser::DataKind::Passive => DataKind::Passive,
            wasmparser::DataKind::Active { memory_index, .. } => DataKind::Active {
                memory_index: *memory_index,
            },
        }
    }

    /// Converts `wasmo` `ElementKind` to `wasmparser` `ElementKind`.
    pub fn to_wasmo_element_kind(ty: &wasmparser::ElementKind) -> ElementKind {
        match ty {
            wasmparser::ElementKind::Passive => ElementKind::Passive,
            wasmparser::ElementKind::Declared => ElementKind::Declared,
            wasmparser::ElementKind::Active { table_index, .. } => ElementKind::Active {
                table_index: *table_index,
            },
        }
    }
}
