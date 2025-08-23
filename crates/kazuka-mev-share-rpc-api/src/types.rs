//! MEV-share bundle type bindings.

use alloy::{
    eips::BlockId,
    primitives::{
        Address, B256, BlockNumber, Bytes, Log, TxHash, U64, U256, address,
        b256, bytes,
    },
};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer, ser::SerializeSeq,
};

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

/// Preferences on what data should be
/// shared about the bundle and its transactions.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Privacy {
    /// Hints on what data should be shared about the bundle and its
    /// transactions.
    pub hints: Option<PrivacyHint>,
    /// Names of the builders that should be allowed to see the
    /// bundle/transaction: https://github.com/flashbots/dowg/blob/main/builder-registrations.json
    pub builders: Option<Vec<String>>,
}

/// Hints on what data should be shared about the bundle and its transactions.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct PrivacyHint {
    /// The calldata of the bundle's transactions should be shared.
    pub calldata: bool,
    /// The address of the bundle's transactions should be shared.
    pub contract_address: bool,
    /// The logs of the bundle's transactions should be shared.
    pub logs: bool,
    /// The function selector of the bundle's transactions should be shared.
    pub function_selector: bool,
    /// The hash of the bundle's transactions should be shared.
    pub hash: bool,
    /// The hash of the bundle should be shared.
    pub tx_hash: bool,
}

impl PrivacyHint {
    pub fn with_calldata(mut self) -> Self {
        self.calldata = true;
        self
    }

    pub fn with_contract_address(mut self) -> Self {
        self.contract_address = true;
        self
    }

    pub fn with_logs(mut self) -> Self {
        self.logs = true;
        self
    }

    pub fn with_function_selector(mut self) -> Self {
        self.function_selector = true;
        self
    }

    pub fn with_hash(mut self) -> Self {
        self.hash = true;
        self
    }

    pub fn with_tx_hash(mut self) -> Self {
        self.tx_hash = true;
        self
    }

    fn num_hints(&self) -> usize {
        let mut num_hints = 0;
        if self.calldata {
            num_hints += 1;
        }
        if self.contract_address {
            num_hints += 1;
        }
        if self.logs {
            num_hints += 1;
        }
        if self.function_selector {
            num_hints += 1;
        }
        if self.hash {
            num_hints += 1;
        }
        if self.tx_hash {
            num_hints += 1;
        }
        num_hints
    }
}

impl Serialize for PrivacyHint {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.num_hints()))?;
        if self.calldata {
            seq.serialize_element("calldata")?;
        }
        if self.contract_address {
            seq.serialize_element("contract_address")?;
        }
        if self.logs {
            seq.serialize_element("logs")?;
        }
        if self.function_selector {
            seq.serialize_element("function_selector")?;
        }
        if self.hash {
            seq.serialize_element("hash")?;
        }
        if self.tx_hash {
            seq.serialize_element("tx_hash")?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for PrivacyHint {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let hints = Vec::<String>::deserialize(deserializer)?;
        let mut privacy_hint = PrivacyHint::default();
        for hint in hints {
            match hint.as_str() {
                "calldata" => privacy_hint.calldata = true,
                "contract_address" => privacy_hint.contract_address = true,
                "logs" => privacy_hint.logs = true,
                "function_selector" => privacy_hint.function_selector = true,
                "hash" => privacy_hint.hash = true,
                "tx_hash" => privacy_hint.tx_hash = true,
                _ => {
                    return Err(serde::de::Error::custom(
                        "invalid privacy hint",
                    ));
                }
            }
        }
        Ok(privacy_hint)
    }
}

/// Used to specify the address and percentage to pay refund from the backrun of
/// this transaction. By default, the refund is paid to the signer of the
/// transaction and 90% of the backrun value is sent to the user by default.
/// If multiple refund addresses are specified, then the backrun value is
/// split between them according to the percentage specified.
///
/// For example, if refund is
/// `[{address: addr1, percent: 10}, {address: addr1, percent: 20}]`
/// then 10% of the backrun value is sent to addr1 and
/// 20% is sent to addr2 and 70% of the backrun value is left to the builder.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Validity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund: Option<Vec<Refund>>,
    /// Specifies what addresses should receive what percent of the overall
    /// refund for this bundle, if it is enveloped by another bundle (e.g. a
    /// searcher backrun). Each entry specifies address that should receive
    /// refund from backrun and percent of the backrun value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_config: Option<Vec<RefundConfig>>,
}

/// The smallest percent of earnings from a bundle that must be shared
/// for the bundle to be part of a builder's block.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Refund {
    /// The index of the transaction in the bundle.
    pub body_idx: u64,
    /// The minimum percent of the bundle's earnings to share.
    pub percent: u64,
}

/// Specifies address that should receive
/// refund from backrun and percent of the backrun value.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundConfig {
    /// Address that should receive refund.
    pub address: Address,
    /// Percentage of the total backrun value that this address should receive.
    pub percent: u64,
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

/// Bundle of transactions for `eth_sendBundle`
///
/// Note: this is for `eth_sendBundle` and not `mev_sendBundle`
///
/// <https://docs.flashbots.net/flashbots-auction/searchers/advanced/rpc-endpoint#eth_sendbundle>
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthSendBundle {
    /// A list of hex-encoded signed transactions.
    pub txs: Vec<Bytes>,
    /// Hex-encoded block number for which this bundle is valid.
    pub block_number: U64,
    /// Unix timestamp when this bundle becomes active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_timestamp: Option<u64>,
    /// Unix timestamp how long this bundle stays valid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_timestamp: Option<u64>,
    /// List of hashes of possibly reverting txs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reverting_tx_hashes: Vec<B256>,
    /// UUID that can be used to cancel/replace this bundle.
    #[serde(
        rename = "replacementUuid",
        skip_serializing_if = "Option::is_none"
    )]
    pub replacement_uuid: Option<String>,
}

/// Response from the matchmaker after sending a bundle.
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EthBundleHash {
    /// Hash of the bundle bodies.
    pub bundle_hash: B256,
}

/// Bundle of transactions for `eth_callBundle`
///
/// <https://docs.flashbots.net/flashbots-auction/searchers/advanced/rpc-endpoint#eth_callBundle>
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthCallBundle {
    /// A list of hex-encoded signed transactions.
    pub txs: Vec<Bytes>,
    /// Hex encoded block number for which this bundle is valid on.
    pub block_number: U64,
    /// Either a hex encoded number or a block tag for which state to base this
    /// simulation on.
    pub state_block_number: BlockNumber,
    /// The timestamp to use for this bundle simulation, in seconds since the
    /// unix epoch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

/// Response for `eth_callBundle`
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EthCallBundleResponse {
    pub bundle_gas_price: U256,
    pub bundle_hash: B256,
    pub coinbase_diff: U256,
    pub eth_sent_to_coinbase: U256,
    pub gas_fees: U256,
    pub results: Vec<EthCallBundleTransactionResult>,
    pub state_block_number: u64,
    pub total_gas_used: u64,
}

/// Result of a single transaction in a bundle for `eth_callBundle`
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthCallBundleTransactionResult {
    pub coinbase_diff: U256,
    pub eth_sent_to_coinbase: U256,
    pub from_address: Address,
    pub gas_fees: U256,
    pub gas_price: U256,
    pub gas_used: u64,
    pub to_address: Address,
    pub tx_hash: B256,
    pub value: Bytes,
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_deserialize_eth_call_response() {
        let expected = EthCallBundleResponse {
            bundle_gas_price: U256::from(476190476193u64),
            bundle_hash: b256!(
                "0x73b1e258c7a42fd0230b2fd05529c5d4b6fcb66c227783f8bece8aeacdd1db2e"
            ),
            coinbase_diff: U256::from(20000000000126000u64),
            eth_sent_to_coinbase: U256::from(20000000000000000u64),
            gas_fees: U256::from(126000u64),
            results: vec![
                EthCallBundleTransactionResult {
                    coinbase_diff: U256::from(10000000000063000u64),
                    eth_sent_to_coinbase: U256::from(10000000000000000u64),
                    from_address: address!(
                        "0x02A727155aeF8609c9f7F2179b2a1f560B39F5A0"
                    ),
                    gas_fees: U256::from(63000u64),
                    gas_price: U256::from(476190476193u64),
                    gas_used: 21000u64,
                    to_address: address!(
                        "0x73625f59CAdc5009Cb458B751b3E7b6b48C06f2C"
                    ),
                    tx_hash: b256!(
                        "0x669b4704a7d993a946cdd6e2f95233f308ce0c4649d2e04944e8299efcaa098a"
                    ),
                    value: bytes!("0x"),
                },
                EthCallBundleTransactionResult {
                    coinbase_diff: U256::from(10000000000063000u64),
                    eth_sent_to_coinbase: U256::from(10000000000000000u64),
                    from_address: address!(
                        "0x02A727155aeF8609c9f7F2179b2a1f560B39F5A0"
                    ),
                    gas_fees: U256::from(63000u64),
                    gas_price: U256::from(476190476193u64),
                    gas_used: 21000,
                    to_address: address!(
                        "0x73625f59CAdc5009Cb458B751b3E7b6b48C06f2C"
                    ),
                    tx_hash: b256!(
                        "0xa839ee83465657cac01adc1d50d96c1b586ed498120a84a64749c0034b4f19fa"
                    ),
                    value: bytes!("0x"),
                },
            ],
            state_block_number: 5221585,
            total_gas_used: 42000,
        };
        let actual_str = r#"{
            "bundleGasPrice": "476190476193",
            "bundleHash": "0x73b1e258c7a42fd0230b2fd05529c5d4b6fcb66c227783f8bece8aeacdd1db2e",
            "coinbaseDiff": "20000000000126000",
            "ethSentToCoinbase": "20000000000000000",
            "gasFees": "126000",
            "results": [
            {
                "coinbaseDiff": "10000000000063000",
                "ethSentToCoinbase": "10000000000000000",
                "fromAddress": "0x02A727155aeF8609c9f7F2179b2a1f560B39F5A0",
                "gasFees": "63000",
                "gasPrice": "476190476193",
                "gasUsed": 21000,
                "toAddress": "0x73625f59CAdc5009Cb458B751b3E7b6b48C06f2C",
                "txHash": "0x669b4704a7d993a946cdd6e2f95233f308ce0c4649d2e04944e8299efcaa098a",
                "value": "0x"
            },
            {
                "coinbaseDiff": "10000000000063000",
                "ethSentToCoinbase": "10000000000000000",
                "fromAddress": "0x02A727155aeF8609c9f7F2179b2a1f560B39F5A0",
                "gasFees": "63000",
                "gasPrice": "476190476193",
                "gasUsed": 21000,
                "toAddress": "0x73625f59CAdc5009Cb458B751b3E7b6b48C06f2C",
                "txHash": "0xa839ee83465657cac01adc1d50d96c1b586ed498120a84a64749c0034b4f19fa",
                "value": "0x"
            }
            ],
            "stateBlockNumber": 5221585,
            "totalGasUsed": 42000
        }"#;

        let actual =
            serde_json::from_str::<EthCallBundleResponse>(actual_str).unwrap();

        assert_eq!(actual, expected);
    }
}
