use alloy::{
    primitives::{Address, B256, BlockNumber, Bytes, U64, U256, bytes},
    rpc::types::mev::{Privacy, Validity},
};
use serde::{Deserialize, Serialize};

/// Bundle of transactions for `eth_sendBundle`
///
/// Note: this is for `eth_sendBundle` and not `mev_sendBundle`
///
/// <https://docs.flashbots.net/flashbots-auction/searchers/advanced/rpc-endpoint#eth_sendbundle>
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendBundleRequest {
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
pub struct BundleHash {
    /// Hash of the bundle bodies.
    pub bundle_hash: B256,
}

/// Bundle of transactions for `eth_callBundle`
///
/// <https://docs.flashbots.net/flashbots-auction/searchers/advanced/rpc-endpoint#eth_callBundle>
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallBundleRequest {
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
pub struct CallBundleResponse {
    pub bundle_gas_price: U256,
    pub bundle_hash: B256,
    pub coinbase_diff: U256,
    pub eth_sent_to_coinbase: U256,
    pub gas_fees: U256,
    pub results: Vec<CallBundleTransactionResult>,
    pub state_block_number: u64,
    pub total_gas_used: u64,
}

/// Result of a single transaction in a bundle for `eth_callBundle`
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallBundleTransactionResult {
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

/// Request for `eth_cancelBundle`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CancelBundleRequest {
    /// Bundle hash of the bundle to be canceled
    pub bundle_hash: String,
}

/// Request for `eth_sendPrivateTransaction`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrivateTransactionRequest {
    /// Raw signed transaction.
    pub tx: Bytes,
    /// Hex-encoded number string, optional.
    /// Highest block number in which the transaction should be included.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block_number: Option<U64>,
    #[serde(
        default,
        skip_serializing_if = "PrivateTransactionPreferences::is_empty"
    )]
    pub preferences: PrivateTransactionPreferences,
}

/// Additional preferences for `eth_sendPrivateTransaction`
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct PrivateTransactionPreferences {
    /// Requirements for the bundle to be included in the block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity: Option<Validity>,
    /// Preferences on what data should be shared about the bundle and its
    /// transactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<Privacy>,
}

impl PrivateTransactionPreferences {
    /// Returns true if the preferences are empty.
    pub fn is_empty(&self) -> bool {
        self.validity.is_none() && self.privacy.is_none()
    }
}

/// Request for `eth_cancelPrivateTransaction`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CancelPrivateTransactionRequest {
    /// Transaction hash of the transaction to be canceled
    pub tx_hash: B256,
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{address, b256};
    #[cfg(test)]
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_deserialize_eth_call_response() {
        let expected = CallBundleResponse {
            bundle_gas_price: U256::from(476190476193u64),
            bundle_hash: b256!(
                "0x73b1e258c7a42fd0230b2fd05529c5d4b6fcb66c227783f8bece8aeacdd1db2e"
            ),
            coinbase_diff: U256::from(20000000000126000u64),
            eth_sent_to_coinbase: U256::from(20000000000000000u64),
            gas_fees: U256::from(126000u64),
            results: vec![
                CallBundleTransactionResult {
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
                CallBundleTransactionResult {
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
            serde_json::from_str::<CallBundleResponse>(actual_str).unwrap();

        assert_eq!(actual, expected);
    }
}
