use std::{collections::HashMap, ops::Add, path::PathBuf, sync::Arc};

use alloy::{
    primitives::{Address, B256, U256},
    providers::Provider,
    rpc::types::mev::{BundleItem, Inclusion, MevSendBundle, ProtocolVersion},
};
use async_trait::async_trait;
use kazuka_core::{error::KazukaError, types::Strategy};
use kazuka_mev_share_arbitrage_bindings::blind_arb::BlindArb::BlindArbInstance;

use crate::{
    contracts::ArbitrageContract,
    types::{Action, Event, UniswapV2PoolInfo, V2V3PoolRecord},
};

pub struct MevShareUniswapV2V3Arbitrage<P: Provider> {
    /// Exposes Ethereum JSON-RPC methods.
    provider: Arc<P>,
    /// Maps Uniswap V3 pool address to Uniswap V2 pool info.
    v3_address_to_v2_pool_info: HashMap<Address, UniswapV2PoolInfo>,
    /// Arbitrage contract.
    contract: ArbitrageContract<Arc<P>>,
}

impl<P: Provider> MevShareUniswapV2V3Arbitrage<P> {
    pub fn new(provider: Arc<P>, arbitrage_contract_address: Address) -> Self {
        let instance = BlindArbInstance::new(
            arbitrage_contract_address,
            provider.clone(),
        );
        let contract = ArbitrageContract::new(provider.clone(), instance);
        Self {
            provider: provider.clone(),
            v3_address_to_v2_pool_info: HashMap::new(),
            contract,
        }
    }

    /// Generates bundles of varying sizes to submit to the matchmaker.
    pub async fn generate_bundles(
        &self,
        v3_address: Address,
        tx_hash: B256,
    ) -> Result<Vec<MevSendBundle>, KazukaError> {
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

        let block_num = self.provider.get_block_number().await?;

        for size in sizes {
            let tx_bytes = self
                .contract
                .generate_arbitrage_tx(v3_address, v2_pool_info, size)
                .await?;

            let bundle_body = vec![
                BundleItem::Hash { hash: tx_hash },
                BundleItem::Tx {
                    tx: tx_bytes,
                    can_revert: false,
                },
            ];

            let bundle = MevSendBundle {
                protocol_version: ProtocolVersion::V0_1,
                inclusion: Inclusion {
                    block: block_num.add(1),
                    // Set a large validity window to ensure builder gets a
                    // chance to include bundle.
                    max_block: Some(block_num.add(30)),
                },
                bundle_body,
                validity: None,
                privacy: None,
            };

            tracing::info!("Constructed bundle: {:?}", bundle);

            bundles.push(bundle);
        }

        Ok(bundles)
    }
}

#[async_trait]
impl<P: Provider> Strategy<Event, Action> for MevShareUniswapV2V3Arbitrage<P> {
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
        match event {
            Event::MevShareEvent(event) => {
                tracing::info!("Received MEV-share event: {:?}", event);
                // Skip if event has no logs.
                if event.logs.is_empty() {
                    return vec![];
                }
                let v3_address = event.logs[0].address;
                // Skip if address is not a V3 pool.
                if !self.v3_address_to_v2_pool_info.contains_key(&v3_address) {
                    return vec![];
                }

                tracing::info!(
                    "Found a V3 pool match at address {:?}, generating bundles",
                    v3_address
                );

                match self.generate_bundles(v3_address, event.hash).await {
                    Ok(bundles) => {
                        bundles.into_iter().map(Action::SubmitBundle).collect()
                    }
                    Err(e) => {
                        tracing::error!("Error generating bundles: {:?}", e);
                        vec![]
                    }
                }
            }
        }
    }
}
