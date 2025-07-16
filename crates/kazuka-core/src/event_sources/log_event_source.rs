use std::sync::Arc;

use alloy::{
    network::AnyNetwork,
    providers::{DynProvider, Provider},
    rpc::types::{Filter, Log},
};
use async_trait::async_trait;

use crate::{
    error::KazukaError,
    types::{EventSource, EventStream},
};

/// Listens for new blockchain event logs based on [Filter](Filter) and
/// generates a stream of [events](Log).
pub struct LogEventSource {
    provider: Arc<DynProvider<AnyNetwork>>,
    filter: Filter,
}

impl LogEventSource {
    pub fn new(provider: Arc<DynProvider<AnyNetwork>>, filter: Filter) -> Self {
        Self { provider, filter }
    }
}

#[async_trait]
impl EventSource<Log> for LogEventSource {
    async fn get_event_stream(
        &self,
    ) -> Result<EventStream<'_, Log>, KazukaError> {
        let subscription = self.provider.subscribe_logs(&self.filter).await?;
        let stream = subscription.into_stream();

        Ok(Box::pin(stream))
    }
}
