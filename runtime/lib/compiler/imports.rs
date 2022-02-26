// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Archive, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Imports {
    pub memories: Vec<Import>,
    pub tables: Vec<Import>,
    pub functions: Vec<Import>,
    pub globals: Vec<Import>,
}

#[derive(Debug, Serialize, Deserialize, Archive, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
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
