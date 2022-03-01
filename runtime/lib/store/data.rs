// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use serde::{Serialize, Deserialize};

/// The `Data` section contains the initial values of the linear memory.
#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub kind: DataKind,
}

/// The kind of data segment.
///
/// https://github.com/WebAssembly/multi-memory/blob/main/proposals/bulk-memory-operations/Overview.md#data-segments
#[derive(Debug, Serialize, Deserialize)]
pub enum DataKind {
    /// Passive represents a data segment that is not initialized by the program.
    Passive,
    /// Active represents a data segment that is initialized by the program.
    ///
    /// `memory_index` is the index of the memory to use.
    Active { memory_index: u32 },
}

impl Data {
    pub fn new(kind: DataKind) -> Self {
        Self { kind }
    }
}
