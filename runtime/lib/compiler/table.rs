use crate::types::{Limits, ValType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Table {
    pub limits: Limits,
    pub element_type: ValType,
}

impl Table {
    pub fn new(limits: Limits, element_type: ValType) -> Self {
        Self {
            limits,
            element_type,
        }
    }
}
