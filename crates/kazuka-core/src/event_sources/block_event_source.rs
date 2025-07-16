use std::sync::Arc;

use alloy::{
    network::AnyNetwork,
    primitives::{BlockHash, BlockNumber, BlockTimestamp},
    providers::{DynProvider, Provider},
};
use async_trait::async_trait;
use tokio_stream::StreamExt;

use crate::{
    error::KazukaError,
    types::{EventSource, EventStream},
};

#[derive(Clone, Debug)]
pub struct NewBlock {
    pub hash: BlockHash,
    pub number: BlockNumber,
    pub timestamp: BlockTimestamp,
}

/// Listens for new blocks, and generates a stream of [events](NewBlock).
pub struct BlockEventSource {
    provider: Arc<DynProvider<AnyNetwork>>,
}

impl BlockEventSource {
    pub fn new(provider: Arc<DynProvider<AnyNetwork>>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl EventSource<NewBlock> for BlockEventSource {
    async fn get_event_stream(
        &self,
    ) -> Result<EventStream<'_, NewBlock>, KazukaError> {
        let subscription = self.provider.subscribe_blocks().await?;
        let stream = subscription.into_stream().map(|header| NewBlock {
            hash: header.hash,
            number: header.number,
            timestamp: header.timestamp,
        });
        Ok(Box::pin(stream))
    }
}
