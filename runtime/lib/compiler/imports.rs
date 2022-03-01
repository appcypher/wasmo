// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use serde::{Serialize, Deserialize};

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
    pub field: Option<String>,
    pub index: u32,
}

impl Import {
    pub fn new(module: String, field: Option<String>, index: u32) -> Self {
        Self {
            module,
            field,
            index,
        }
    }
}
