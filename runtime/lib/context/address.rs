use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BaseAddress {
    // pub address: T, // TODO(appcypher): Make this machine-dependent
}
