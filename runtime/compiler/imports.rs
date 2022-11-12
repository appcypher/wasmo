use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Imports {
    pub memories: Vec<Import>,
    pub tables: Vec<Import>,
    pub functions: Vec<Import>,
    pub globals: Vec<Import>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Import {
    pub module: String,
    pub name: String,
    pub index: u32,
}

impl Import {
    pub fn new(module: String, name: String, index: u32) -> Self {
        Self { module, name, index }
    }
}
