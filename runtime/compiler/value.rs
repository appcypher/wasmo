use serde::{Deserialize, Serialize};

/// TODO(appcypher): Document this.
#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    Num(NumVal),
    Ref(RefVal),
    Vec(i128),
}

/// TODO(appcypher): Document this.
#[derive(Debug, Serialize, Deserialize)]
pub enum NumVal {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

/// TODO(appcypher): Document this.
#[derive(Debug, Serialize, Deserialize)]
pub enum RefVal {
    FuncAddr(i32),
    ExternAddr(i64),
}
