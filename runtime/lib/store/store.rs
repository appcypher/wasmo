// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use serde::{Serialize, Deserialize};

use crate::context::BaseAddress;

/// Store maintains maintains pointers to data sections representing the entire global state of a WebAssembly
#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    // Local Memories
    pub memories_section: BaseAddress,
    pub tables_section: BaseAddress,
    pub globals_section: BaseAddress,
    pub elems_section: BaseAddress,
    pub data_section: BaseAddress,
}
