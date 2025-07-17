use std::{sync::Arc, time::Duration};

use alloy::{
    consensus::Transaction,
    eips::BlockId,
    network::{AnyNetwork, TransactionBuilder},
    primitives::U256,
    providers::{DynProvider, Provider, ProviderBuilder, WsConnect},
    rpc::types::TransactionRequest,
    serde::WithOtherFields,
};
use alloy_node_bindings::{Anvil, AnvilInstance};
use futures::StreamExt;
use kazuka_core::{
    event_sources::{
        block_event_source::BlockEventSource,
        mempool_event_source::MempoolEventSource,
    },
    executors::mempool_executor::{MempoolExecutor, SubmitTxToMempool},
    types::{EventSource, Executor},
};
use tokio::time::sleep;

/// Spawns Anvil and instantiates a WebSocket provider.
pub async fn spawn_anvil() -> (DynProvider<AnyNetwork>, AnvilInstance) {
    let anvil = Anvil::new().block_time(1).spawn();
    let ws = WsConnect::new(anvil.ws_endpoint_url());
    let provider = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .connect_ws(ws)
        .await
        .unwrap();

    let provider = DynProvider::new(provider);
    (provider, anvil)
}

/// Test that block event source correctly emits blocks.
#[tokio::test]
async fn test_block_event_source_emits_blocks() {
    let (provider, _anvil) = spawn_anvil().await;
    let provider = Arc::new(provider);
    let block_event_source = BlockEventSource::new(Arc::clone(&provider));
    let block_steam = block_event_source.get_event_stream().await.unwrap();
    let block_a = block_steam.into_future().await.0.unwrap();
    let block_b = provider
        .get_block(BlockId::latest())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(block_a.hash, block_b.header.hash);
}

/// Test that mempool event source correctly emits blocks.
#[tokio::test]
async fn test_mempool_event_source_emits_txs() {
    let (provider, _anvil) = spawn_anvil().await;
    let provider = Arc::new(provider);
    let mempool_event_source = MempoolEventSource::new(Arc::clone(&provider));
    let mempool_stream = mempool_event_source.get_event_stream().await.unwrap();

    let alice_address = provider.get_accounts().await.unwrap()[0];
    let bob_address = provider.get_accounts().await.unwrap()[1];

    let value = U256::from(42);
    let gas_price = 100000000000000000_u128;
    let tx = TransactionRequest::default()
        .with_from(alice_address)
        .with_to(bob_address)
        .with_value(value)
        .with_gas_price(gas_price);

    let _ = provider
        .send_transaction(WithOtherFields::new(tx))
        .await
        .unwrap();

    let emitted_tx = mempool_stream.into_future().await.0.unwrap();

    assert_eq!(emitted_tx.value(), value);
}

/// Test that the mempool executor correctly sends txs.
#[tokio::test]
async fn test_mempool_executor_sends_tx() {
    let (provider, _anvil) = spawn_anvil().await;
    let provider = Arc::new(provider);
    let mempool_executor = MempoolExecutor::new(Arc::clone(&provider));

    let alice_address = provider.get_accounts().await.unwrap()[0];
    let bob_address = provider.get_accounts().await.unwrap()[1];

    let value = U256::from(42);
    let gas_price = 100000000000000000_u128;
    let tx = TransactionRequest::default()
        .with_from(alice_address)
        .with_to(bob_address)
        .with_value(value)
        .with_gas_price(gas_price);

    let action = SubmitTxToMempool {
        tx: WithOtherFields::new(tx),
        gas_bid_info: None,
    };

    mempool_executor.execute(action).await.unwrap();

    // Sleep 2 seconds so that the tx has time to be mined.
    sleep(Duration::from_secs(2)).await;

    let count = provider.get_transaction_count(alice_address).await.unwrap();
    assert_eq!(count, 1);
}
