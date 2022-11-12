use serde::{Deserialize, Serialize};

use crate::types::Limits;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Memory {
    pub is_memory_64: bool, // TODO(appcypher): Wasmo does not support memory64 proposal yet.
    pub is_shared: bool,
    pub limits: Limits,
}

impl Memory {
    pub fn new(limits: Limits, is_shared: bool) -> Self {
        Self {
            is_memory_64: false,
            is_shared,
            limits,
        }
    }
}
