// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

/// WebAssembly function type as defined in the spec.
///
/// https://webassembly.github.io/spec/core/syntax/types.html#syntax-functype
#[derive(Debug, Serialize, Deserialize, Archive)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct FuncType {
    pub params: Vec<ValType>,
    pub returns: Vec<ValType>,
}

/// WebAssembly value types as defined in the spec.
///
/// https://webassembly.github.io/spec/core/syntax/types.html#syntax-valtype
#[derive(Debug, Serialize, Deserialize, Archive)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub enum ValType {
    NumType(NumType), // i32, i64, f32, f64
    RefType(RefType), // funcref, externref
    VecType,          // v128
}

/// WebAssembly num types as defined in the spec.
///
/// https://webassembly.github.io/spec/core/syntax/types.html#syntax-numtype
#[derive(Debug, Serialize, Deserialize, Archive)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

/// WebAssembly num types as defined in the spec.
///
/// https://webassembly.github.io/spec/core/syntax/types.html#syntax-reftype
#[derive(Debug, Serialize, Deserialize, Archive)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub enum RefType {
    FuncRef,
    ExternRef,
}

/// WebAssembly limits almost as defined in the spec.
///
/// A slight deviation from the current spec. Wasmo uses 64-bit types as there will be support for memory64 in the future.
///
/// https://webassembly.github.io/spec/core/syntax/types.html#syntax-limits
#[derive(Debug, Serialize, Deserialize, Archive, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Limits {
    /// Intial page count.
    pub min: u64,
    /// Maximum page count.
    pub max: Option<u64>,
}

/// Webassembly memory and table page size.
/// 64KiB.
pub const _PAGE_SIZE: u32 = 65536;

impl Limits {
    pub fn new(min: u64, max: Option<u64>) -> Self {
        Self { min, max }
    }
}
