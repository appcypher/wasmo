use serde::{Deserialize, Serialize};

/// Store manages the entire global state accessible to a WebAssembly instance.
#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    // Local Memories
}
