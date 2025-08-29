use alloy::primitives::{B256, U64};
use serde::{Deserialize, Serialize};

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
