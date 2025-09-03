use core::fmt;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use alloy::rpc::types::mev::mevshare::{
    EventHistory, EventHistoryInfo, EventHistoryParams,
};
use async_sse::Decoder;
use bytes::Bytes;
use futures_util::{
    Stream, TryFutureExt, TryStreamExt,
    future::BoxFuture,
    ready,
    stream::{IntoAsyncRead, MapErr, MapOk},
};
use http::{HeaderValue, header};
use pin_project_lite::pin_project;
use serde::{Serialize, de::DeserializeOwned};
use tracing::{instrument, trace};

use crate::Event;

/// The client for SSE.
///
/// This is a simple wrapper around [reqwest::Client] that provides subscription
/// function for SSE.
#[derive(Debug, Clone)]
pub struct EventClient {
    reqwest_client: reqwest::Client,
    max_retries: Option<u64>,
}

impl Default for EventClient {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl EventClient {
    /// Creates a new client with the given reqwest client.
    ///
    /// ```
    /// use kazuka_mev_share_sse::EventClient;
    /// let client = EventClient::new(reqwest::Client::new());
    /// ```
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            reqwest_client: client,
            max_retries: None,
        }
    }

    /// Sets the maximum number of retries.
    pub fn with_max_retries(mut self, max_retries: u64) -> Self {
        self.set_max_retries(max_retries);
        self
    }

    /// Sets the maximum number of retries.
    pub fn set_max_retries(&mut self, max_retries: u64) {
        self.max_retries = Some(max_retries)
    }

    /// Returns the maximum number of retries.
    pub fn max_retries(&self) -> Option<u64> {
        self.max_retries
    }

    /// Subscribe to the MEV-share SSE endpoint.
    ///
    /// This connects to the endpoint and returns a stream of `T` items.
    ///
    /// See [EventClient::events] for a more convenient way to subscribe to
    /// [Event] streams.
    #[instrument(name = "MEV-share SSE subscribing", skip(self))]
    pub async fn subscribe<T: DeserializeOwned + fmt::Debug>(
        &self,
        endpoint: &str,
    ) -> reqwest::Result<EventStream<T>> {
        let stream = ActiveEventStream::<T>::connect(
            &self.reqwest_client,
            endpoint,
            None::<()>,
        )
        .await?;

        let endpoint = endpoint.to_string();
        let inner = EventStreamInner {
            num_retries: 0,
            endpoint,
            event_client: self.clone(),
            query: None,
        };
        let state = Some(State::Active(Box::pin(stream)));
        Ok(EventStream { inner, state })
    }

    /// Subscribe to the MEV-share SSE endpoint with additional query params.
    /// This connects to the endpoint and returns a stream of `T` items.
    ///
    /// See [EventClient::events] for a more convenient way to subscribe to
    /// [Event] streams.
    #[instrument(
        name = "MEV-share SSE subscribing with query",
        skip(self, query)
    )]
    pub async fn subscribe_with_query<
        T: DeserializeOwned + fmt::Debug,
        S: Serialize,
    >(
        &self,
        endpoint: &str,
        query: S,
    ) -> reqwest::Result<EventStream<T>> {
        let query =
            Some(serde_json::to_value(query).expect("Serialization failed"));
        let stream = ActiveEventStream::<T>::connect(
            &self.reqwest_client,
            endpoint,
            query.as_ref(),
        )
        .await?;
        let endpoint = endpoint.to_string();
        let inner = EventStreamInner {
            num_retries: 0,
            endpoint,
            event_client: self.clone(),
            query: None,
        };
        let state = Some(State::Active(Box::pin(stream)));
        Ok(EventStream { inner, state })
    }

    /// Subscribe to a stream of [Event]s.
    /// This is a convenience function for [EventClient::subscribe].
    pub async fn events(
        &self,
        endpoint: &str,
    ) -> reqwest::Result<EventStream<Event>> {
        self.subscribe(endpoint).await
    }

    /// Gets past events that were broadcast via the SSE event stream.
    ///
    /// Such as `https://mev-share.flashbots.net/api/v1/history`.
    pub async fn event_history(
        &self,
        endpoint: &str,
        params: EventHistoryParams,
    ) -> reqwest::Result<Vec<EventHistory>> {
        self.reqwest_client
            .get(endpoint)
            .query(&params)
            .send()
            .await?
            .json()
            .await
    }

    /// Gets information about the event history endpoint
    ///
    /// Such as `https://mev-share.flashbots.net/api/v1/history/info`.
    pub async fn event_history_info(
        &self,
        endpoint: &str,
    ) -> reqwest::Result<Vec<EventHistoryInfo>> {
        self.reqwest_client.get(endpoint).send().await?.json().await
    }
}

/// A stream of SSE items.
#[must_use = "streams do nothing unless polled"]
pub struct EventStream<T: fmt::Debug> {
    inner: EventStreamInner,
    state: Option<State<T>>,
}

impl<T: fmt::Debug> EventStream<T> {
    /// The endpoint this stream is connected to.
    pub fn endpoint(&self) -> &str {
        &self.inner.endpoint
    }

    /// Resets all retry attempts.
    pub fn reset_retries(&mut self) {
        self.inner.num_retries = 0;
    }
}

impl<T: DeserializeOwned + fmt::Debug> EventStream<T> {
    /// Retries the stream by establishing a new connection.
    #[instrument(name = "MEV-share SSE retring", skip(self))]
    pub async fn retry(&mut self) -> Result<(), SseError> {
        let stream = self.inner.retry().await?;
        self.state = Some(State::Active(Box::pin(stream)));
        Ok(())
    }

    /// Retries the stream by establishing a new connection using the given
    /// endpoint.
    #[instrument(
        name = "MEV-share SSE retrying with new endpoint",
        skip(self, endpoint)
    )]
    pub async fn retry_with(
        &mut self,
        endpoint: impl Into<String>,
    ) -> Result<(), SseError> {
        self.inner.endpoint = endpoint.into();
        let stream = self.inner.retry().await?;
        self.state = Some(State::Active(Box::pin(stream)));
        Ok(())
    }
}

impl<T: fmt::Debug> fmt::Debug for EventStream<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventStream")
            .field("endpoint", &self.inner.endpoint)
            .field("num_retries", &self.inner.num_retries)
            .field(
                "client",
                &self.inner.event_client.reqwest_client,
            )
            .finish_non_exhaustive()
    }
}

impl<T: DeserializeOwned + fmt::Debug> Stream for EventStream<T> {
    type Item = Result<T, SseError>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let mut result = Poll::Pending;

        loop {
            let state = this
                .state
                .take()
                .expect("EventStream polled after completion");

            match state {
                // Stream has finished.
                State::End => {
                    tracing::debug!("state = end");
                    return Poll::Ready(None);
                }
                // Currently retrying, poll the future, which might resolve to a
                // new ActiveEventStream or an SseError.
                State::Retry(mut future) => {
                    tracing::debug!("state = retry");
                    match future.as_mut().poll(cx) {
                        Poll::Ready(Ok(stream)) => {
                            tracing::debug!(
                                "successfully retried, reconnected, got a new stream"
                            );
                            this.state = Some(State::Active(Box::pin(stream)));
                            tracing::debug!("continue polling");
                            continue;
                        }
                        Poll::Ready(Err(err)) => {
                            tracing::debug!(
                                "failed to retry, stopping, returning error"
                            );
                            this.state = Some(State::End);
                            return Poll::Ready(Some(Err(err)));
                        }
                        Poll::Pending => {
                            tracing::debug!("still pending retry");
                            this.state = Some(State::Retry(future));
                            return Poll::Pending;
                        }
                    }
                }
                // Already connected, poll the currently active stream.
                State::Active(mut stream) => {
                    tracing::debug!("state = active");
                    match stream.as_mut().poll_next(cx) {
                        Poll::Ready(None) => {
                            tracing::debug!("active stream finished, stopping");
                            this.state = Some(State::End);
                            return Poll::Ready(None);
                        }
                        Poll::Ready(Some(Ok(event_or_retry))) => {
                            tracing::debug!("active stream ready");

                            match event_or_retry {
                                // Got an event - return it.
                                EventOrRetry::Event(event) => {
                                    tracing::debug!(?event, "got event");
                                    result = Poll::Ready(Some(Ok(event)));
                                }
                                // Got a retry -
                                // start retrying after the duration.
                                EventOrRetry::Retry(duration) => {
                                    tracing::debug!("got retry");
                                    let mut client = this.inner.clone();
                                    let future = Box::pin(async move {
                                        tokio::time::sleep(duration).await;
                                        client.retry().await
                                    });
                                    this.state = Some(State::Retry(future));
                                    continue;
                                }
                            }
                        }
                        Poll::Ready(Some(Err(err))) => {
                            // Got an error, end the stream.
                            tracing::warn!(?err, "active stream error");
                            result = Poll::Ready(Some(Err(err)));
                        }
                        Poll::Pending => {
                            tracing::debug!("active stream pending");
                        }
                    }
                    this.state = Some(State::Active(stream));
                    break;
                }
            }
        }

        result
    }
}

/// State machine for [EventStream].
enum State<T: fmt::Debug> {
    /// Stream has finished.
    End,
    /// Waiting for retry future to resolve.
    Retry(BoxFuture<'static, Result<ActiveEventStream<T>, SseError>>),
    /// Active, connected stream.
    Active(Pin<Box<ActiveEventStream<T>>>),
}

/// Inner state of [EventStream].
#[derive(Clone)]
pub struct EventStreamInner {
    /// Number of retries.
    num_retries: u64,
    /// Endpoint to connect to.
    endpoint: String,
    /// Client to use for connecting.
    event_client: EventClient,
    /// Query parameters..
    query: Option<serde_json::Value>,
}

impl EventStreamInner {
    /// Retries the stream by creating a new subscription stream.
    #[instrument(name = "MEV-share SSE retrying", skip(self))]
    async fn retry<T: DeserializeOwned + fmt::Debug>(
        &mut self,
    ) -> Result<ActiveEventStream<T>, SseError> {
        self.num_retries += 1;

        if let Some(max_retries) = self.event_client.max_retries
            && self.num_retries > max_retries
        {
            return Err(SseError::MaxRetriesExceeded(
                max_retries,
            ));
        }
        tracing::debug!(
            retries = self.num_retries,
            "retrying SSE stream"
        );
        ActiveEventStream::connect(
            &self.event_client.reqwest_client,
            &self.endpoint,
            self.query.as_ref(),
        )
        .map_err(SseError::RetryError)
        .await
    }
}

type ToIoError = fn(reqwest::Error) -> std::io::Error;
type ToEventOrRetry<T> =
    fn(async_sse::Event) -> serde_json::Result<EventOrRetry<T>>;

type RequestStream = Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send>>;

type SseDecoderStream<T> = MapOk<
    Decoder<IntoAsyncRead<MapErr<RequestStream, ToIoError>>>,
    ToEventOrRetry<T>,
>;

enum EventOrRetry<T: fmt::Debug> {
    Retry(Duration),
    Event(T),
}

pin_project! {
    struct ActiveEventStream<T: fmt::Debug> {
        #[pin]
        stream: SseDecoderStream<T>,
    }
}

impl<T: DeserializeOwned + fmt::Debug> Stream for ActiveEventStream<T> {
    type Item = Result<EventOrRetry<T>, SseError>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match ready!(this.stream.poll_next(cx)) {
            None => Poll::Ready(None),
            Some(result) => {
                let item = match result {
                    Ok(Ok(ev)) => Ok(ev),
                    Ok(Err(err)) => Err(SseError::SerdeJsonError(err)),
                    Err(err) => Err(SseError::Http(err)),
                };
                Poll::Ready(Some(item))
            }
        }
    }
}

impl<T> ActiveEventStream<T>
where
    T: DeserializeOwned + fmt::Debug,
{
    /// Connects to the SSE endpoint and returns a new [ActiveEventStream].
    #[instrument(name = "MEV-share SSE connecting", skip(client, query))]
    async fn connect<S: Serialize>(
        client: &reqwest::Client,
        endpoint: &str,
        query: Option<S>,
    ) -> reqwest::Result<ActiveEventStream<T>> {
        let mut builder = client
            .get(endpoint)
            .header(
                header::ACCEPT,
                HeaderValue::from_static("text/event-stream"),
            )
            .header(
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-cache"),
            );

        if let Some(query) = query {
            builder = builder.query(&query);
        }

        let response = builder.send().await?;

        // Converts reqwest errors to io::Error.
        let to_io_error: ToIoError = std::io::Error::other;

        // Converts SSE events to [EventOrRetry].
        let to_event_or_retry: ToEventOrRetry<_> = |event| match event {
            async_sse::Event::Message(message) => {
                trace!(message = ?String::from_utf8_lossy(message.data()), "received message");
                serde_json::from_slice::<T>(message.data())
                    .map(EventOrRetry::Event)
            }
            async_sse::Event::Retry(duration) => {
                trace!(?duration, "receive retry");
                Ok(EventOrRetry::Retry(duration))
            }
        };

        let event_stream: RequestStream = Box::pin(response.bytes_stream());
        let reader = event_stream.map_err(to_io_error).into_async_read();
        let stream = async_sse::decode(reader).map_ok(to_event_or_retry);

        Ok(ActiveEventStream { stream })
    }
}

/// Error variants that can occur while handling an SSE subscription.
#[derive(Debug, thiserror::Error)]
pub enum SseError {
    /// Failed to deserialize the SSE event data.
    #[error("Failed to deserialize event: {0}")]
    SerdeJsonError(serde_json::Error),
    /// Http error.
    #[error("{0}")]
    Http(http_types::Error),
    /// Failed to establish a retry connection.
    #[error("Failed to establish a retry connection: {0}")]
    RetryError(reqwest::Error),
    /// Exceeded all retries.
    #[error("Exceeded all retries: {0}")]
    MaxRetriesExceeded(u64),
}
