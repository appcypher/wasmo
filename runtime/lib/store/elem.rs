// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::context::Address;

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Archive)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Element {
    pub address: Option<Address>,
    pub kind: ElementKind,
}

#[derive(Debug, Serialize, Deserialize, Archive)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub enum ElementKind {
    Passive,
    Active { table_index: u32 },
    Declared,
}

impl Element {
    pub fn new(kind: ElementKind) -> Self {
        Self {
            kind,
            address: None,
        }
    }
}
