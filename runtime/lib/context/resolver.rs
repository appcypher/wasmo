// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

/// The resolver is responsible for resolving the addresses of referenced components of a module.
/// 
#[derive(Debug, Serialize, Deserialize, Archive, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct CompileTimeResolver {} // TODO(appcypher)
