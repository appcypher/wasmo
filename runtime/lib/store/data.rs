// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::context::Address;

use super::LocalityKind;

#[derive(Debug)]
pub struct Data {
    pub kind: LocalityKind,
    pub address: Address,
}
