// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.
use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

/// The different options for configuring the runtime.
#[derive(Debug, Serialize, Deserialize, Archive, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Options {
    /// Whether to use the Liftoff compiler.
    pub liftoff: bool,
}
