//! MEV-share event type bindings.

// {
//     hash: string,
//     logs?: LogParams[],
//     txs: Array<{
//         hash?: string,
//         callData?: string,
//         functionSelector?: string,
//         to?: string,
//         from?: string,
//         value?: string,
//         maxFeePerGas?: string,
//         maxPriorityFeePerGas?: string,
//         nonce?: string,
//         chainId?: string,
//         accessList?: Array<{
//             address: string,
//             storageKeys: string[]
//         }>,
//         gas?: string,
//         type?: string
//     }>
// }

use alloy::{
    primitives::{Address, Bytes, TxHash, U256},
    rpc::types::mev::mevshare::{EventTransactionLog, FunctionSelector},
};
use num_traits::Num;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

/// SSE event from the MEV-share endpoint.
/// See: https://docs.flashbots.net/flashbots-mev-share/searchers/event-stream#event-scheme
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Event {
    /// Transaction or bundle hash.
    pub hash: TxHash,
    /// Event logs emitted by executing the transaction.
    #[serde(with = "null_sequence")]
    pub logs: Vec<EventTransactionLog>,

    /// Transactions from the event. If the event itself is a transaction, txs
    /// will only have one entry. Bundle events may have more.
    #[serde(rename = "txs", with = "null_sequence")]
    pub transactions: Vec<EventTransaction>,
}

/// Transaction from the MEV-share event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventTransaction {
    /// Transaction hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<TxHash>,
    /// Calldata of the transaction
    #[serde(rename = "callData")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calldata: Option<Bytes>,
    /// 4-byte-function selector
    #[serde(rename = "functionSelector")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_selector: Option<FunctionSelector>,
    /// Transaction recipient address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,
    /// Transaction sender address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// Transaction value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
    /// Maximum fee per gas.
    #[serde(rename = "maxFeePerGas")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256>,
    /// Maximum priority fee per gas.
    #[serde(rename = "maxPriorityFeePerGas")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>,
    /// Transaction nonce.
    #[serde(deserialize_with = "hex_to_option_unsigned")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,
    /// Transaction chain ID.
    #[serde(rename = "chainId")]
    #[serde(deserialize_with = "hex_to_option_unsigned")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
    /// Transaction access list.
    #[serde(rename = "accessList")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<Vec<AccessListEntry>>,
    /// Transaction gas limit.
    #[serde(deserialize_with = "hex_to_option_unsigned")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<u64>,
    /// Transaction type.
    #[serde(rename = "type")]
    #[serde(deserialize_with = "hex_to_option_unsigned")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_type: Option<u64>,
}

/// Contains address and storage slots accessed by transaction.
/// See: <https://rareskills.io/post/eip-2930-optional-access-list-ethereum>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccessListEntry {
    pub address: Address,
    #[serde(rename = "storageKeys")]
    pub storage_keys: Vec<U256>,
}

/// Deserializes missing or null sequences as empty vectors.
mod null_sequence {
    use serde::{
        Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned,
    };

    pub(crate) fn deserialize<'de, D, T>(
        deserializer: D,
    ) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: DeserializeOwned,
    {
        let s =
            Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default();
        Ok(s)
    }

    pub(crate) fn serialize<T, S>(
        val: &Vec<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        if val.is_empty() {
            serializer.serialize_none()
        } else {
            val.serialize(serializer)
        }
    }
}

fn hex_to_option_unsigned<'de, D, T>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Num + Copy,
    <T as Num>::FromStrRadixErr: std::fmt::Display,
{
    let opt: Option<&str> = Option::deserialize(deserializer)?;
    if let Some(s) = opt {
        let s = s
            .strip_prefix("0x")
            .ok_or_else(|| D::Error::custom("missing 0x prefix"))?;
        let val = T::from_str_radix(s, 16).map_err(D::Error::custom)?;
        Ok(Some(val))
    } else {
        Ok(None)
    }
}
