// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Exports {
    pub(crate) inner: HashMap<String, Export>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Export {
    pub kind: ExportKind,
    pub index: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExportKind {
    Memory,
    Table,
    Function,
    Global,
}

impl Export {
    pub fn new(kind: ExportKind, index: u32) -> Self {
        Self { kind, index }
    }
}
