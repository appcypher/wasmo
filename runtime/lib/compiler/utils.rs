// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

pub mod convert {
    use crate::{
        errors::CompilerError,
        types::{FuncType, NumType, RefType, ValType},
    };
    use utilities::result::Result;

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
            wasmparser::Type::FuncRef => Ok(ValType::RefType(RefType::Func)),
            wasmparser::Type::ExternRef => Ok(ValType::RefType(RefType::Extern)),
            unsupported => Err(CompilerError::UnsupportedValType(unsupported.clone()).into()),
        }
    }
}
