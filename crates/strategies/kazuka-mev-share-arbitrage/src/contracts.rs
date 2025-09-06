use alloy::{
    network::TransactionBuilder,
    primitives::{Address, Bytes, U256},
    providers::Provider,
    sol,
};
use kazuka_core::error::KazukaError;
use kazuka_mev_share_arbitrage_bindings::blind_arb::BlindArb::BlindArbInstance;

use crate::types::UniswapV2PoolInfo;

sol!(
    BlindArb,
    "./contracts/out/BlindArb.sol/BlindArb.json"
);

/// Wrapper to simplify working with `BlindArbInstance`.
pub(crate) struct ArbitrageContract<P: Provider> {
    provider: P,
    instance: BlindArbInstance<P>,
}

impl<P: Provider> ArbitrageContract<P> {
    pub(crate) fn new(provider: P, instance: BlindArbInstance<P>) -> Self {
        Self { provider, instance }
    }

    pub(crate) async fn generate_arbitrage_tx(
        &self,
        v3_address: Address,
        v2_pool_info: &UniswapV2PoolInfo,
        size: U256,
    ) -> Result<Bytes, KazukaError> {
        // Set parameters for backruns.
        let payment_percentage = U256::ZERO;
        let bid_gas_price = self.provider.get_gas_price().await?;

        let mut tx = if v2_pool_info.is_weth_token0 {
            self.instance
                .execute_weth_token0(
                    v2_pool_info.v2_pool,
                    v3_address,
                    size,
                    payment_percentage,
                )
                .into_transaction_request()
        } else {
            self.instance
                .execute_weth_token1(
                    v2_pool_info.v2_pool,
                    v3_address,
                    size,
                    payment_percentage,
                )
                .into_transaction_request()
        };
        tx.set_gas_limit(400000);
        tx.set_gas_price(bid_gas_price);

        tracing::info!(
            "Generated arbitrage transaction: {:?}",
            tx
        );

        let tx_bytes = self.provider.sign_transaction(tx).await?;
        Ok(tx_bytes)
    }
}
