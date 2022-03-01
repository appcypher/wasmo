// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use serde::{Deserialize, Serialize};

/// Store manages the entire global state accessible to a WebAssembly instance.
#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    // Local Memories
}
