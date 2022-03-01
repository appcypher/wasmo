// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use serde::{Serialize, Deserialize};
use crate::types::{Limits, ValType};

#[derive(Debug, Serialize, Deserialize)]
pub struct Table {
    pub limits: Limits,
    pub element_type: ValType,
}

impl Table {
    pub fn new(limits: Limits, element_type: ValType) -> Self {
        Self {
            limits,
            element_type,
        }
    }
}
