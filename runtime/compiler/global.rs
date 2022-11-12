use serde::{Deserialize, Serialize};

use crate::types::ValType;

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
