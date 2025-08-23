use alloy::primitives::{B256, U64, U256};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

/// Response for `flashbots_getUserStatsV2` represents stats for a searcher.
///
/// Note: this is V2: <https://docs.flashbots.net/flashbots-auction/searchers/advanced/rpc-endpoint#flashbots_getuserstatsv2>
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    /// Represents whether this searcher has a high enough reputation to be in
    /// the high priority queue.
    pub is_high_priority: bool,
    /// The total amount paid to validators over all time.
    pub all_time_validator_payments: U256,
    /// The total amount of gas simulated across all bundles submitted to
    /// Flashbots. This is the actual gas used in simulations, not gas
    /// limit.
    pub all_time_gas_simulated: U256,
    /// The total amount paid to validators the last 7 days.
    pub last_7d_validator_payments: U256,
    /// The total amount of gas simulated across all bundles submitted to
    /// Flashbots in the last 7 days. This is the actual gas used in
    /// simulations, not gas limit.
    pub last_7d_gas_simulated: U256,
    /// The total amount paid to validators the last day.
    pub last_1d_validator_payments: U256,
    /// The total amount of gas simulated across all bundles submitted to
    /// Flashbots in the last day. This is the actual gas used in
    /// simulations, not gas limit.
    pub last_1d_gas_simulated: U256,
}

/// Response for `flashbots_getBundleStatsV2` represents stats for a single
/// bundle.
///
/// Note: this is V2: <https://docs.flashbots.net/flashbots-auction/searchers/advanced/rpc-endpoint#flashbots_getbundlestatsv2>
///
/// Timestamp format: "2022-10-06T21:36:06.322Z"
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum BundleStats {
    /// The relayer has not yet seen the bundle.
    #[default]
    Unknown,
    /// The relayer has seen the bundle, but has not simulated it yet.
    Seen(StatsSeen),
    /// The relayer has seen the bundle and has simulated it.
    Simulated(StatsSimulated),
}

impl Serialize for BundleStats {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match self {
            BundleStats::Unknown => {
                serde_json::json!({"isSimulated": false}).serialize(serializer)
            }
            BundleStats::Seen(stats) => stats.serialize(serializer),
            BundleStats::Simulated(stats) => stats.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for BundleStats {
    fn deserialize<D>(deserializer: D) -> Result<BundleStats, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = serde_json::Map::deserialize(deserializer)?;

        if map.get("receivedAt").is_none() {
            Ok(BundleStats::Unknown)
        } else if map["isSimulated"] == false {
            StatsSeen::deserialize(serde_json::Value::Object(map))
                .map(BundleStats::Seen)
                .map_err(serde::de::Error::custom)
        } else {
            StatsSimulated::deserialize(serde_json::Value::Object(map))
                .map(BundleStats::Simulated)
                .map_err(serde::de::Error::custom)
        }
    }
}

/// Response for `flashbots_getBundleStatsV2` represents stats for a single
/// bundle.
///
/// Note: this is V2: <https://docs.flashbots.net/flashbots-auction/searchers/advanced/rpc-endpoint#flashbots_getbundlestatsv2>
///
/// Timestamp format: "2022-10-06T21:36:06.322Z
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsSeen {
    /// boolean representing if this searcher has a high enough reputation to
    /// be in the high priority queue
    pub is_high_priority: bool,
    /// representing whether the bundle gets simulated. All other fields will
    /// be omitted except simulated field if API didn't receive bundle
    pub is_simulated: bool,
    /// time at which the bundle API received the bundle
    pub received_at: String,
}

/// Response for `flashbots_getBundleStatsV2` represents stats for a single
/// bundle.
///
/// Note: this is V2: <https://docs.flashbots.net/flashbots-auction/searchers/advanced/rpc-endpoint#flashbots_getbundlestatsv2>
///
/// Timestamp format: "2022-10-06T21:36:06.322Z
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsSimulated {
    /// boolean representing if this searcher has a high enough reputation to
    /// be in the high priority queue
    pub is_high_priority: bool,
    /// representing whether the bundle gets simulated. All other fields will
    /// be omitted except simulated field if API didn't receive bundle
    pub is_simulated: bool,
    /// time at which the bundle gets simulated
    pub simulated_at: String,
    /// time at which the bundle API received the bundle
    pub received_at: String,
    /// indicates time at which each builder selected the bundle to be included
    /// in the target block
    #[serde(default = "Vec::new")]
    pub considered_by_builders_at: Vec<ConsideredByBuildersAt>,
    /// indicates time at which each builder sealed a block containing the
    /// bundle
    #[serde(default = "Vec::new")]
    pub sealed_by_builders_at: Vec<SealedByBuildersAt>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsideredByBuildersAt {
    pub pubkey: String,
    pub timestamp: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SealedByBuildersAt {
    pub pubkey: String,
    pub timestamp: String,
}
