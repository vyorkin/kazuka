use alloy::{
    eips::BlockId,
    primitives::{Address, B256, Bytes, Log, TxHash, U64, bytes},
};
use serde::{Deserialize, Serialize};

use crate::types::core::{Privacy, Validity};

/// A bundle of transactions to send to the matchmaker.
///
/// Note: this is for `mev_sendBundle` and not `eth_sendBundle`.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SendBundleRequest {
    /// The version of the MEV-share API to use.
    #[serde(rename = "version")]
    pub protocol_version: ProtocolVersion,
    /// Data used by block builders to check if
    /// the bundle should be considered for inclusion.
    #[serde(rename = "inclusion")]
    pub inclusion: Inclusion,
    /// The transactions to include in the bundle.
    #[serde(rename = "body")]
    pub bundle_body: Vec<BundleItem>,
    /// Requirements for the bundle to be included in the block.
    #[serde(rename = "validity", skip_serializing_if = "Option::is_none")]
    pub validity: Option<Validity>,
    /// What data should be shared about the bundle and its transactions.
    #[serde(rename = "privacy", skip_serializing_if = "Option::is_none")]
    pub privacy: Option<Privacy>,
}

impl SendBundleRequest {
    pub fn new(
        block_num: U64,
        max_block: Option<U64>,
        protocol_version: ProtocolVersion,
        bundle_body: Vec<BundleItem>,
    ) -> Self {
        Self {
            protocol_version,
            inclusion: Inclusion {
                block: block_num,
                max_block,
            },
            bundle_body,
            validity: None,
            privacy: None,
        }
    }
}

/// Response from the matchmaker after sending a bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendBundleResponse {
    /// Hash of the bundle bodies.
    pub bundle_hash: B256,
}

/// Optional fields to override simulation state.
#[derive(Deserialize, Debug, Serialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SimBundleOverrides {
    /// Block used for simulation state. Defaults to latest block.
    /// Block header data will be derived from parent block by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_block: Option<BlockId>,
    /// Block number used for simulation, defaults to parentBlock.number + 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<U64>,
    /// Coinbase used for simulation, defaults to parentBlock.coinbase
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coinbase: Option<Address>,
    /// Timestamp used for simulation, defaults to parentBlock.timestamp + 12
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<U64>,
    /// Gas limit used for simulation, defaults to parentBlock.gasLimit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_limit: Option<U64>,
    /// Base fee used for simulation, defaults to parentBlock.baseFeePerGas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_fee: Option<U64>,
    /// Timeout in seconds, defaults to 5
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<U64>,
}

/// Response from the matchmaker after sending a simulation request.
#[derive(Deserialize, Debug, Default, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SimBundleResponse {
    /// Whether the simulation was successful.
    pub success: bool,
    /// Error message if the simulation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The block number of the simulated block.
    pub state_block: U64,
    /// The gas price of the simulated block.
    pub mev_gas_price: U64,
    /// The profit of the simulated block.
    pub profit: U64,
    /// The refundable value of the simulated block.
    pub refundable_value: U64,
    /// The gas used by the simulated block.
    pub gas_used: U64,
    /// Logs returned by mev_simBundle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<SimBundleLogs>>,
}

/// Logs returned by mev_simBundle.
#[derive(Deserialize, Debug, Default, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SimBundleLogs {
    /// Logs for transactions in bundle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_logs: Option<Vec<Log>>,
    /// Logs for bundles in bundle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_logs: Option<Vec<SimBundleLogs>>,
}

/// A bundle tx, which can either be a transaction hash, or a full tx.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum BundleItem {
    /// The hash of either a transaction or bundle we are trying to backrun.
    Hash { hash: TxHash },
    /// A new signed transaction.
    #[serde(rename_all = "camelCase")]
    Tx {
        /// Bytes of the signed transaction.
        tx: Bytes,
        /// If true, the transaction can revert
        /// without the bundle being considered invalid.
        can_revert: bool,
    },
}

/// The version of the MEV-share API to use.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolVersion {
    /// The beta-1 version of the MEV-share API.
    #[default]
    #[serde(rename = "beta-1")]
    Beta1,
    /// The 0.1 version of the MEV-share API.
    #[serde(rename = "v0.1")]
    V0_1,
}

/// Data used by block builders to check if
/// the bundle should be considered for inclusion.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inclusion {
    /// The first block the bundle is valid for.
    pub block: U64,
    /// The last block the bundle is valid for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block: Option<U64>,
}

impl Inclusion {
    /// Creates a new inclusion with the given min block.
    pub fn at_block(block: u64) -> Self {
        Self {
            block: U64::from(block),
            max_block: None,
        }
    }

    /// Returns the block number of the first block the bundle is valid for.
    #[inline]
    pub fn block_number(&self) -> u64 {
        self.block.to::<u64>()
    }

    /// Returns the block number of the last block the bundle is valid for.
    #[inline]
    pub fn max_block_number(&self) -> Option<u64> {
        self.max_block.as_ref().map(|b| b.to::<u64>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::core::{PrivacyHint, RefundConfig};

    #[test]
    fn test_deserialize_simple() {
        let str = r#"
        [{
            "version": "v0.1",
            "inclusion": {
                "block": "0x1"
            },
            "body": [{
                "tx": "0x02f86b0180843b9aca00852ecc889a0082520894c87037874aed04e51c29f582394217a0a2b89d808080c080a0a463985c616dd8ee17d7ef9112af4e6e06a27b071525b42182fe7b0b5c8b4925a00af5ca177ffef2ff28449292505d41be578bebb77110dfc09361d2fb56998260",
                "canRevert": false
            }]
        }]
        "#;

        let result: Result<Vec<SendBundleRequest>, _> =
            serde_json::from_str(str);
        assert!(result.is_ok())
    }

    #[test]
    fn test_deserialize_complex() {
        let str = r#"
        [{
            "version": "v0.1",
            "inclusion": {
                "block": "0x1"
            },
            "body": [{
                "tx": "0x02f86b0180843b9aca00852ecc889a0082520894c87037874aed04e51c29f582394217a0a2b89d808080c080a0a463985c616dd8ee17d7ef9112af4e6e06a27b071525b42182fe7b0b5c8b4925a00af5ca177ffef2ff28449292505d41be578bebb77110dfc09361d2fb56998260",
                "canRevert": false
            }],
            "privacy": {
                "hints": [
                  "calldata"
                ]
              },
              "validity": {
                "refundConfig": [
                  {
                    "address": "0x8EC1237b1E80A6adf191F40D4b7D095E21cdb18f",
                    "percent": 100
                  }
                ]
              }
        }]
        "#;

        let result: Result<Vec<SendBundleRequest>, _> =
            serde_json::from_str(str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_serialize_complex() {
        let expected_str = r#"
        [{
            "version": "v0.1",
            "inclusion": {
                "block": "0x1"
            },
            "body": [{
                "tx": "0x02f86b0180843b9aca00852ecc889a0082520894c87037874aed04e51c29f582394217a0a2b89d808080c080a0a463985c616dd8ee17d7ef9112af4e6e06a27b071525b42182fe7b0b5c8b4925a00af5ca177ffef2ff28449292505d41be578bebb77110dfc09361d2fb56998260",
                "canRevert": false
            }],
            "privacy": {
                "hints": [
                  "calldata"
                ]
              },
              "validity": {
                "refundConfig": [
                  {
                    "address": "0x8EC1237b1E80A6adf191F40D4b7D095E21cdb18f",
                    "percent": 100
                  }
                ]
              }
        }]
        "#;

        let bundle_body = vec![BundleItem::Tx {
            tx: bytes!(
                "0x02f86b0180843b9aca00852ecc889a0082520894c87037874aed04e51c29f582394217a0a2b89d808080c080a0a463985c616dd8ee17d7ef9112af4e6e06a27b071525b42182fe7b0b5c8b4925a00af5ca177ffef2ff28449292505d41be578bebb77110dfc09361d2fb56998260"
            ),
            can_revert: false,
        }];

        let validity = Validity {
            refund_config: Some(vec![RefundConfig {
                address: "0x8EC1237b1E80A6adf191F40D4b7D095E21cdb18f"
                    .parse()
                    .unwrap(),
                percent: 100,
            }]),
            ..Default::default()
        };

        let privacy = Privacy {
            hints: Some(PrivacyHint {
                calldata: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        let bundle = SendBundleRequest {
            protocol_version: ProtocolVersion::V0_1,
            inclusion: Inclusion {
                block: U64::from(1),
                max_block: None,
            },
            bundle_body,
            validity: Some(validity),
            privacy: Some(privacy),
        };

        let expected =
            serde_json::from_str::<Vec<SendBundleRequest>>(expected_str)
                .unwrap();

        assert_eq!(bundle, expected[0]);
    }

    #[test]
    fn test_serialize_privacy_hint() {
        let hint = PrivacyHint {
            calldata: true,
            contract_address: true,
            logs: true,
            function_selector: true,
            hash: true,
            tx_hash: true,
        };
        let expected = r#"["calldata","contract_address","logs","function_selector","hash","tx_hash"]"#;
        let actual = serde_json::to_string(&hint).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_deserialize_privacy_hint() {
        let expected = PrivacyHint {
            calldata: true,
            contract_address: false,
            logs: true,
            function_selector: false,
            hash: true,
            tx_hash: false,
        };
        let actual_str = r#"["calldata","logs","hash"]"#;
        let actual: PrivacyHint = serde_json::from_str(actual_str).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_deserialize_sim_bundle_response() {
        let expected = SimBundleResponse {
            success: true,
            error: None,
            state_block: U64::from(0x8b8da8u64),
            mev_gas_price: U64::from(0x74c7906005u64),
            profit: U64::from(0x4bc800904fc000u64),
            refundable_value: U64::from(0x4bc800904fc000u64),
            gas_used: U64::from(0xa620),
            logs: Some(vec![
                Default::default(),
                Default::default(),
            ]),
        };
        let actual_str = r#"
        {
            "success": true,
            "stateBlock": "0x8b8da8",
            "mevGasPrice": "0x74c7906005",
            "profit": "0x4bc800904fc000",
            "refundableValue": "0x4bc800904fc000",
            "gasUsed": "0xa620",
            "logs": [{},{}]
          }
        "#;
        let actual: SimBundleResponse =
            serde_json::from_str(actual_str).unwrap();
        assert_eq!(actual, expected);
    }
}
