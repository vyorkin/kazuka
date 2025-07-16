use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use tokio_stream::StreamExt;

use crate::error::KazukaError;

/// A stream of events emitted by a [EventSource](EventSource).
pub type EventStream<'a, E> = Pin<Box<dyn Stream<Item = E> + Send + 'a>>;

/// Takes in external events (pending txs, new blocks,
/// marketplace orders, etc.) and turns them into events.
#[async_trait]
pub trait EventSource<E>: Send + Sync {
    async fn get_event_stream(&self)
    -> Result<EventStream<'_, E>, KazukaError>;
}

/// Wraps [EventSource](EventSource) and
/// maps outgoing events to a different type.
pub struct EventSourceMap<E, F> {
    event_source: Box<dyn EventSource<E>>,
    f: F,
}

impl<E, F> EventSourceMap<E, F> {
    pub fn new(event_source: Box<dyn EventSource<E>>, f: F) -> Self {
        Self { event_source, f }
    }
}

#[async_trait]
impl<E1, E2, F> EventSource<E2> for EventSourceMap<E1, F>
where
    E1: Send + Sync + 'static,
    E2: Send + Sync + 'static,
    F: Fn(E1) -> E2 + Send + Sync + Clone + 'static,
{
    async fn get_event_stream(
        &self,
    ) -> Result<EventStream<'_, E2>, KazukaError> {
        let stream = self.event_source.get_event_stream().await?;
        let f = self.f.clone();
        let stream = stream.map(f);
        Ok(Box::pin(stream))
    }
}

/// Executes actions returned by [Strategy](Strategy).
#[async_trait]
pub trait Executor<A>: Send + Sync {
    async fn execute(&self, action: A) -> Result<(), KazukaError>;
}

/// Wraps [Executor](Executor) and maps incoming actions to a different type.
pub struct ExecutorMap<A, F> {
    executor: Box<dyn Executor<A>>,
    f: F,
}

impl<A, F> ExecutorMap<A, F> {
    pub fn new(executor: Box<dyn Executor<A>>, f: F) -> Self {
        Self { executor, f }
    }
}

#[async_trait]
impl<A1, A2, F> Executor<A1> for ExecutorMap<A2, F>
where
    A1: Send + Sync + 'static,
    A2: Send + Sync + 'static,
    F: Fn(A1) -> Option<A2> + Send + Sync + Clone + 'static,
{
    async fn execute(&self, action: A1) -> Result<(), KazukaError> {
        let action = (self.f)(action);
        match action {
            Some(action) => self.executor.execute(action).await,
            None => Ok(()),
        }
    }
}

/// Contains the core logic required for each MEV opportunity.
/// They take in events as inputs, and compute whether any opportunities are
/// available. Strategies produce actions.
#[async_trait]
pub trait Strategy<E, A>: Send + Sync {
    /// Sync the initial state of the strategy if needed, usually by fetching
    /// onchain data.
    async fn sync_state(&mut self) -> Result<(), KazukaError>;

    /// Process an event, and return an action if needed.
    async fn process_event(&mut self, event: E) -> Vec<A>;
}

#[derive(Debug)]
pub enum Event {
    NewBlock,
    Transaction,
}

#[derive(Debug)]
pub enum Action {
    SubmitTxToMempool,
}

#[cfg(test)]
mod tests {
    use futures::stream;

    use super::*;

    struct MockEventSource;

    #[async_trait]
    impl EventSource<Event> for MockEventSource {
        async fn get_event_stream(
            &self,
        ) -> Result<EventStream<'_, Event>, KazukaError> {
            let events = vec![Event::NewBlock, Event::Transaction];
            let stream = stream::iter(events);
            Ok(Box::pin(stream))
        }
    }

    #[tokio::test]
    async fn test_event_source_map() {
        let src: Box<dyn EventSource<Event>> = Box::new(MockEventSource);
        let map = EventSourceMap::new(src, |e: Event| match e {
            Event::NewBlock => "block".to_string(),
            Event::Transaction => "transaction".to_string(),
        });

        let stream = map
            .get_event_stream()
            .await
            .expect("EventSourceMap didn't return event stream");

        let events: Vec<_> = stream.collect().await;

        assert_eq!(
            events,
            vec!["block".to_string(), "transaction".to_string()]
        )
    }
}
