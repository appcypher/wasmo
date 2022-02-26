// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Archive, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Exports {
    pub memories: Vec<Export>,
    pub tables: Vec<Export>,
    pub functions: Vec<Export>,
    pub globals: Vec<Export>,
}

#[derive(Debug, Serialize, Deserialize, Archive, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Export {
    pub field: String,
    pub index: u32,
}

impl Export {
    pub fn new(field: String, index: u32) -> Self {
        Self { field, index }
    }
}
