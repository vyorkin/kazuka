use alloy::primitives::B256;
use serde::{Deserialize, Serialize};

/// Response from the matchmaker after sending a bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendBundleResponse {
    /// Hash of the bundle bodies.
    pub bundle_hash: B256,
}
