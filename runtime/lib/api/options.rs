use serde::{Deserialize, Serialize};

/// The different options for configuring the runtime.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Options {
    /// Whether to use the Liftoff compiler.
    pub liftoff: bool,
}
