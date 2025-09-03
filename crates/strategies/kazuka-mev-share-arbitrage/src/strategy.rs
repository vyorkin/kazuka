use std::{collections::HashMap, path::PathBuf, sync::Arc};

use alloy::{
    network::AnyNetwork,
    primitives::{Address, B256, U256},
    providers::{DynProvider, Provider},
    rpc::types::mev::MevSendBundle,
    signers::local::PrivateKeySigner,
    sol,
};
use async_trait::async_trait;
use kazuka_core::{error::KazukaError, types::Strategy};

use crate::types::{Action, Event, V2V3PoolRecord};

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
    /// Exposes Ethereum JSON-RPC methods.
    provider: Arc<DynProvider<AnyNetwork>>,
    /// Signer for transactions.
    signer: PrivateKeySigner,
    /// Maps Uniswap V3 pool address to Uniswap V2 pool info.
    v3_address_to_v2_pool_info: HashMap<Address, UniswapV2PoolInfo>,
}

impl MevShareUniswapV2V3Arbitrage {
    pub fn new(
        provider: Arc<DynProvider<AnyNetwork>>,
        signer: PrivateKeySigner,
    ) -> Self {
        Self {
            provider: provider.clone(),
            signer,
            v3_address_to_v2_pool_info: HashMap::new(),
        }
    }

    pub async fn generate_bundles(
        &self,
        v3_address: Address,
        tx_hash: B256,
    ) -> Vec<MevSendBundle> {
        let mut bundles = Vec::new();

        // The sizes of the backruns we want to submit.
        // TODO: Run some analysis to figure out likely sizes.
        let sizes = vec![
            U256::from(100000_u128),
            U256::from(1000000_u128),
            U256::from(10000000_u128),
            U256::from(100000000_u128),
            U256::from(1000000000_u128),
            U256::from(10000000000_u128),
            U256::from(100000000000_u128),
            U256::from(1000000000000_u128),
            U256::from(10000000000000_u128),
            U256::from(100000000000000_u128),
            U256::from(1000000000000000_u128),
            U256::from(10000000000000000_u128),
            U256::from(100000000000000000_u128),
            U256::from(1000000000000000000_u128),
        ];

        let v2_pool_info = self
            .v3_address_to_v2_pool_info
            .get(&v3_address)
            .expect("Failed to get V3 pool info");

        // Set parameters for backruns
        let payment_percentage = U256::from(0);
        let bid_gas_price = self
            .provider
            .get_gas_price()
            .await
            .expect("Failed to get gas price");
        let block_num = self
            .provider
            .get_block_number()
            .await
            .expect("Failed to get the last block number");

        for size in sizes {}

        bundles
    }
}

#[async_trait]
impl Strategy<Event, Action> for MevShareUniswapV2V3Arbitrage {
    /// Syncs the initial state of the strategy.
    /// This is called once at startup, and loads pool information into memory.
    async fn sync_state(&mut self) -> Result<(), KazukaError> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("data/uniswap_v2_uniswap_v3_weth_pools.csv");

        let mut reader = csv::Reader::from_path(path)
            .map_err(|e| KazukaError::CsvError(e.to_string()))?;

        for record in reader.deserialize() {
            let record: V2V3PoolRecord =
                record.map_err(|e| KazukaError::CsvError(e.to_string()))?;
            self.v3_address_to_v2_pool_info.insert(
                record.v3_pool,
                UniswapV2PoolInfo {
                    v2_pool: record.v2_pool,
                    is_weth_token0: record.is_weth_token0,
                },
            );
        }

        Ok(())
    }

    /// Processes a MEV-share event, and return an action if needed.
    async fn process_event(&mut self, event: Event) -> Vec<Action> {
        todo!()
    }
}
