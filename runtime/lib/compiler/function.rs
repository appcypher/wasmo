use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Function {
    pub type_index: u32,
}

impl Function {
    pub fn new(type_index: u32) -> Self {
        Self { type_index }
    }
}
