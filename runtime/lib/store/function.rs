// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::context::Address;

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Archive, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Function {
    pub type_index: u32,
    pub address: Option<Address>,
}

impl Function {
    pub fn new(type_index: u32) -> Self {
        Self {
            type_index,
            address: None,
        }
    }
}
