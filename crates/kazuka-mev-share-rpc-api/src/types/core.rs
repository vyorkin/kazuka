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
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Refund {
    /// The index of the transaction in the bundle.
    pub body_idx: u64,
    /// The minimum percent of the bundle's earnings to share.
    pub percent: u64,
}

/// Specifies address that should receive
/// refund from backrun and percent of the backrun value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundConfig {
    /// Address that should receive refund.
    pub address: Address,
    /// Percentage of the total backrun value that this address should receive.
    pub percent: u64,
}

/// Preferences on what data should be
/// shared about the bundle and its transactions.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
