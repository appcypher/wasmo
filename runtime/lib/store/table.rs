// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use super::LocalityKind;
use crate::{context::Address, types::Limits};

pub struct Table {
    pub limits: Limits,
    pub kind: LocalityKind,
    pub address: Address,
}
