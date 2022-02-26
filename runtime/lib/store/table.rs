// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::{
    context::Address,
    types::{Limits, ValType},
};
use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Archive)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Table {
    pub limits: Limits,
    pub element_type: ValType,
    pub address: Option<Address>,
}

impl Table {
    pub fn new(limits: Limits, element_type: ValType) -> Self {
        Self {
            limits,
            element_type,
            address: None,
        }
    }
}
