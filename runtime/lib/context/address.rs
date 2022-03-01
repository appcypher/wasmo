// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BaseAddress {
    // pub address: T, // TODO(appcypher): Make this machine-dependent
}
