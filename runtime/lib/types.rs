use serde::{Deserialize, Serialize};

/// WebAssembly function type as defined in the spec.
///
/// https://webassembly.github.io/spec/core/syntax/types.html#syntax-functype
#[derive(Debug, Serialize, Deserialize)]
pub struct FuncType {
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,
}

/// WebAssembly value types as defined in the spec.
///
/// https://webassembly.github.io/spec/core/syntax/types.html#syntax-valtype
#[derive(Debug, Serialize, Deserialize)]
pub enum ValType {
    Num(NumType), // i32, i64, f32, f64
    Ref(RefType), // funcref, externref
    Vec,          // v128
}

/// WebAssembly num types as defined in the spec.
///
/// https://webassembly.github.io/spec/core/syntax/types.html#syntax-numtype
#[derive(Debug, Serialize, Deserialize)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

/// WebAssembly num types as defined in the spec.
///
/// https://webassembly.github.io/spec/core/syntax/types.html#syntax-reftype
#[derive(Debug, Serialize, Deserialize)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

/// WebAssembly limits almost as defined in the spec.
///
/// A slight deviation from the current spec. Wasmo uses 64-bit types as there will be support for memory64 in the future.
///
/// https://webassembly.github.io/spec/core/syntax/types.html#syntax-limits
#[derive(Debug, Serialize, Deserialize, Default)]
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
