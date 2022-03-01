// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::types::ValType;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Global {
    pub content_type: ValType,
    pub is_mutable: bool,
}

impl Global {
    pub fn new(content_type: ValType, is_mutable: bool) -> Self {
        Self {
            content_type,
            is_mutable,
        }
    }
}
