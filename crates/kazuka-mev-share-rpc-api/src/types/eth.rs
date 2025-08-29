use alloy::primitives::B256;
use serde::{Deserialize, Serialize};

/// Response from the matchmaker after sending a bundle.
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BundleHash {
    /// Hash of the bundle bodies.
    pub bundle_hash: B256,
}
