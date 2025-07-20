use async_trait::async_trait;
use tokio_stream::StreamExt;

use crate::{
    error::KazukaError,
    types::{EventSource, EventStream},
};

pub type MevShareEvent = mev_share::sse::Event;

/// Streams from MEV-Share SSE endpoint and
/// generates [events](Event), which return tx hash, logs, and bundled txs.
pub struct MevShareEventSource {
    mevshare_sse_url: String,
}

impl MevShareEventSource {
    pub fn new(mevshare_sse_url: String) -> Self {
        Self { mevshare_sse_url }
    }
}

#[async_trait]
impl EventSource<MevShareEvent> for MevShareEventSource {
    async fn get_event_stream(
        &self,
    ) -> Result<EventStream<'_, MevShareEvent>, KazukaError> {
        let client = mev_share::sse::EventClient::default();
        let stream = client
            .events(&self.mevshare_sse_url)
            .await
            .expect("Expected MEV-Share SSE stream")
            .filter_map(|event| event.ok());
        Ok(Box::pin(stream))
    }
}
