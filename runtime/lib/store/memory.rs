// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::{context::Address, types::Limits};
use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Archive, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Memory {
    pub is_memory_64: bool, // TODO(appcypher): Wasmo does not support memory64 proposal yet.
    pub is_shared: bool,
    pub limits: Limits,
    pub address: Option<Address>,
}

impl Memory {
    pub fn new(limits: Limits, is_shared: bool) -> Self {
        Self {
            is_memory_64: false,
            is_shared,
            limits,
            address: None,
        }
    }
}
