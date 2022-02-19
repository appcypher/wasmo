// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use rkyv::{Serialize, Deserialize, Archive};
use bytecheck::CheckBytes;

#[derive(Debug, Serialize, Deserialize, Archive)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Instance {}

impl Instance {
}
