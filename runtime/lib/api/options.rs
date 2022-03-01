// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use serde::{Deserialize, Serialize};

/// The different options for configuring the runtime.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Options {
    /// Whether to use the Liftoff compiler.
    pub liftoff: bool,
}
