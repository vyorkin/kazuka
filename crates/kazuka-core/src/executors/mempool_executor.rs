use std::{
    ops::{Div, Mul},
    sync::Arc,
};

use alloy::{
    network::{AnyNetwork, TransactionBuilder},
    primitives::U128,
    providers::{DynProvider, Provider},
    rpc::types::TransactionRequest,
    serde::WithOtherFields,
};
use async_trait::async_trait;

use crate::{error::KazukaError, types::Executor};

pub struct MempoolExecutor {
    provider: Arc<DynProvider<AnyNetwork>>,
}

impl MempoolExecutor {
    pub fn new(provider: Arc<DynProvider<AnyNetwork>>) -> Self {
        Self { provider }
    }
}

#[derive(Clone, Debug)]
pub struct GasBidInfo {
    /// Expected total profit from the opportunity (in wei).
    pub expected_profit: U128,
    /// Fraction of profit you want to give to the miner, as a percentage
    /// (e.g., 50 means 50%).
    pub bid_percentage: U128,
}

#[derive(Clone, Debug)]
pub struct SubmitTxToMempool {
    pub tx: WithOtherFields<TransactionRequest>,
    pub gas_bid_info: Option<GasBidInfo>,
}

#[async_trait]
impl Executor<SubmitTxToMempool> for MempoolExecutor {
    /// Send a transaction to the mempool.
    async fn execute(
        &self,
        action: SubmitTxToMempool,
    ) -> Result<(), KazukaError> {
        let mut tx = action.tx.clone();
        // Expected actual gas usage for the transaction.
        let gas_usage = self.provider.estimate_gas(action.tx).await?;

        let bid_gas_price: U128;
        if let Some(gas_bid_info) = action.gas_bid_info {
            // Gas price at which we'd break even, meaning 100% of profit goes
            // to validator (your entire profit will be spent on gas).
            // This is the maximum gas price you can set without going negative.
            let breakeven_gas_price: U128 =
                gas_bid_info.expected_profit / U128::from(gas_usage);
            // Calculate the actual bid gas price as a fraction of the profit.
            bid_gas_price = breakeven_gas_price
                .mul(U128::from(gas_bid_info.bid_percentage))
                .div(U128::from(100));

            // Example:
            //
            // expected_profit = 0.02 ETH = 20_000_000_000_000_000 wei
            // gas_usage = estimate_gas(tx) = 200_000 gas units
            // bid_percentage = 40
            //
            // breakeven_gas_price = expected_profit / gas_usage =
            // 20_000_000_000_000_000 / 200_000 = 100_000_000_000 wei
            //
            // bid_gas_price = (breakeven_gas_price / 100) * bid_percentage =
            // (100_000_000_000 / 100) * 40 =
            // 100_000_000_000 * 40 / 100 =
            // 40_000_000_000 wei
            //
            // If you set the gas price at 40 gwei, you give 40% of your profit
            // to the validator and keep 60% yourself.
            // If you set the gas price at 100 gwei, you give the entire profit
            // to the validator (you keep zero).
        } else {
            // Otherwise use market gas price.
            bid_gas_price = U128::from(self.provider.get_gas_price().await?);
        }

        tx.set_gas_price(bid_gas_price.to());
        let _ = self.provider.send_transaction(tx).await?;
        Ok(())
    }
}
