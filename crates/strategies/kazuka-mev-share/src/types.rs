use alloy::primitives::Address;
use mev_share::{rpc::SendBundleRequest, sse};

#[derive(Clone, Debug)]
pub enum Event {
    MevShareEvent(sse::Event),
}

#[derive(Clone, Debug)]
pub enum Action {
    // Submit a bundle of transactions to the matchmaker.
    SubmitBundle(SendBundleRequest),
}

#[derive(Debug, serde::Deserialize)]
pub struct PoolRecord {
    pub token_address: Address,
    pub uni_pool_address: Address,
    pub sushi_pool_address: Address,
}

#[derive(Debug, serde::Deserialize)]
pub struct V2V3PoolRecord {
    pub token_address: Address,
    pub v2_pool: Address,
    pub v3_pool: Address,
    pub is_weth_token0: bool,
}
