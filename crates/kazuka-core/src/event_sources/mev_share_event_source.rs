use async_trait::async_trait;
use kazuka_mev_share::sse;
use tokio_stream::StreamExt;

use crate::{
    error::KazukaError,
    types::{EventSource, EventStream},
};

pub type MevShareEvent = kazuka_mev_share::sse::Event;

/// Streams from MEV-Share SSE endpoint and
/// generates [events](Event), which return tx hash, logs, and bundled txs.
pub struct MevShareEventSource {
    mev_share_sse_url: String,
}

impl MevShareEventSource {
    pub fn new(url: String) -> Self {
        Self {
            mev_share_sse_url: url,
        }
    }
}

#[async_trait]
impl EventSource<MevShareEvent> for MevShareEventSource {
    async fn get_event_stream(
        &self,
    ) -> Result<EventStream<'_, MevShareEvent>, KazukaError> {
        let client = sse::EventClient::default();
        let stream = client
            .events(&self.mev_share_sse_url)
            .await
            .expect("Expected MEV-Share SSE stream")
            .filter_map(Result::ok);
        Ok(Box::pin(stream))
    }
}
