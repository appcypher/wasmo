// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Element {
    pub kind: ElementKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ElementKind {
    Passive,
    Active { table_index: u32 },
    Declared,
}

impl Element {
    pub fn new(kind: ElementKind) -> Self {
        Self { kind }
    }
}
