use std::collections::HashMap;

use alloy::{primitives::Address, signers::local::PrivateKeySigner, sol};
use async_trait::async_trait;
use kazuka_core::{error::KazukaError, types::Strategy};

use crate::types::{Action, Event};

sol!(
    BlindArb,
    "./contracts/out/BlindArb.sol/BlindArb.json"
);

#[derive(Clone, Debug)]
pub struct UniswapV2PoolInfo {
    /// Address of the Uniswap V2 pool.
    pub v2_pool: Address,
    /// Whether the pool has weth as token0.
    pub is_weth_token0: bool,
}

#[derive(Clone, Debug)]
pub struct MevShareUniswapV2V3Arbitrage {
    /// Maps Uniswap V3 pool address to Uniswap V2 pool info.
    v3_address_to_v2_pool_info: HashMap<Address, UniswapV2PoolInfo>,
    /// Signer for transactions.
    signer: PrivateKeySigner,
}

impl MevShareUniswapV2V3Arbitrage {
    pub fn new(signer: PrivateKeySigner) -> Self {
        Self {
            signer,
            v3_address_to_v2_pool_info: HashMap::new(),
        }
    }
}

#[async_trait]
impl Strategy<Event, Action> for MevShareUniswapV2V3Arbitrage {
    // Syncs the initial state of the strategy.
    async fn sync_state(&mut self) -> Result<(), KazukaError> {
        todo!();
        Ok(())
    }

    /// Processes a MEV-share event, and return an action if needed.
    async fn process_event(&mut self, event: Event) -> Vec<Action> {
        todo!()
    }
}
