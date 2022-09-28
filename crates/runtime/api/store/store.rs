use serde::{Deserialize, Serialize};

/// Store manages the entire global state accessible to a WebAssembly instance.
#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    // Imported Memories
    // Imported Tables
    // Imported Globals
    // Local Memories
    // Local Tables
    // Local Globals

    // Imported Functions
    // Intrinsics
    // Version
}
