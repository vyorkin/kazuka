//! MEV-share bundle type bindings.

use alloy::primitives::{B256, U64};
use serde::{Deserialize, Serialize};

/// Response from the matchmaker after sending a bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendBundleResponse {
    /// Hash of the bundle bodies.
    pub bundle_hash: B256,
}

/// Response from the matchmaker after sending a bundle.
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BundleHash {
    /// Hash of the bundle bodies.
    pub bundle_hash: B256,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetUserStatsRequest {
    pub block_number: U64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBundleStatsRequest {
    pub bundle_hash: B256,
    pub block_number: U64,
}
