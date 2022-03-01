// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    I32(u32),
    I64(u64),
    F32(f32),
    F64(f64),
}
