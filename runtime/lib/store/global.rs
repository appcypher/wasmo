// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::{context::Address, types::ValType};
use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Archive)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Global {
    pub content_type: ValType,
    pub is_mutable: bool,
    pub address: Option<Address>,
}

impl Global {
    pub fn new(content_type: ValType, is_mutable: bool) -> Self {
        Self {
            content_type,
            is_mutable,
            address: None,
        }
    }
}
