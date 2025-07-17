use std::sync::Arc;

use alloy::{
    network::{AnyNetwork, AnyRpcTransaction},
    providers::{DynProvider, Provider},
};
use async_trait::async_trait;
use futures::StreamExt;

use crate::{
    error::KazukaError,
    types::{EventSource, EventStream},
};

/// Listens for new transactions in the mempool, and
/// generates a stream of [events](Transaction).
pub struct MempoolEventSource {
    provider: Arc<DynProvider<AnyNetwork>>,
}

impl MempoolEventSource {
    pub fn new(provider: Arc<DynProvider<AnyNetwork>>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl EventSource<AnyRpcTransaction> for MempoolEventSource {
    async fn get_event_stream(
        &self,
    ) -> Result<EventStream<'_, AnyRpcTransaction>, KazukaError> {
        let subscription = self
            .provider
            .subscribe_pending_transactions()
            .await
            .inspect_err(|e| {
                tracing::error!(
                    "Error subscribing to pending transactions: {}",
                    e
                );
            })?;

        let provider = Arc::clone(&self.provider);
        let stream = subscription.into_stream().filter_map(move |hash| {
            let provider = Arc::clone(&provider);
            async move {
                provider
                    .get_transaction_by_hash(hash)
                    .await
                    .inspect_err(|e| {
                        tracing::error!(
                            "Error getting transaction by hash: {}",
                            e
                        )
                    })
                    .ok()
                    .flatten()
            }
        });

        Ok(Box::pin(stream))
    }
}
