use std::fmt::Debug;

use tokio::{
    sync::broadcast::{self, Sender},
    task::JoinSet,
};
use tokio_stream::StreamExt;

use crate::{
    error::KazukaError,
    types::{EventSource, Executor, Strategy},
};

const DEFAULT_CHANNEL_CAPACITY: usize = 512;

pub struct Engine<E, A> {
    event_sources: Vec<Box<dyn EventSource<E>>>,
    strategies: Vec<Box<dyn Strategy<E, A>>>,
    executors: Vec<Box<dyn Executor<A>>>,

    event_channel_capacity: usize,
    action_channel_capacity: usize,
}

impl<E, A> Engine<E, A> {
    pub fn new() -> Self {
        Self {
            event_sources: vec![],
            strategies: vec![],
            executors: vec![],
            event_channel_capacity: DEFAULT_CHANNEL_CAPACITY,
            action_channel_capacity: DEFAULT_CHANNEL_CAPACITY,
        }
    }
}

impl<E, A> Default for Engine<E, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E, A> Engine<E, A>
where
    E: Send + Clone + 'static + Debug,
    A: Send + Clone + 'static + Debug,
{
    pub fn add_event_source(mut self, source: Box<dyn EventSource<E>>) -> Self {
        self.event_sources.push(source);
        self
    }

    pub fn add_strategy(mut self, strategy: Box<dyn Strategy<E, A>>) -> Self {
        self.strategies.push(strategy);
        self
    }

    pub fn add_executor(mut self, executor: Box<dyn Executor<A>>) -> Self {
        self.executors.push(executor);
        self
    }

    /// The core run loop of the engine.
    /// This function will spawn a thread for each collector, strategy, and
    /// executor. It will then orchestrate the data flow between them.
    pub async fn run(self) -> Result<JoinSet<()>, KazukaError> {
        let (event_sender, _): (Sender<E>, _) =
            broadcast::channel(self.event_channel_capacity);
        let (action_sender, _): (Sender<A>, _) =
            broadcast::channel(self.action_channel_capacity);

        let mut tasks = JoinSet::new();

        for executor in self.executors {
            let mut receiver = action_sender.subscribe();
            tasks.spawn(async move {
                tracing::info!("Starting executor...");
                loop {
                    match receiver.recv().await {
                        Ok(action) => match executor.execute(action).await {
                            Ok(()) => {}
                            Err(e) => {
                                tracing::error!("Error executing action: {}", e)
                            }
                        },
                        Err(e) => {
                            tracing::error!("Error receiving action: {}", e)
                        }
                    }
                }
            });
        }

        for mut strategy in self.strategies {
            let mut event_receiver = event_sender.subscribe();
            let action_sender = action_sender.clone();
            tracing::info!("Syncing strategy's state...");
            strategy.sync_state().await?;
            tasks.spawn(async move {
                tracing::info!("Starting strategy...");
                loop {
                    match event_receiver.recv().await {
                        Ok(event) => {
                            let actions = strategy.process_event(event).await;
                            for action in actions {
                                match action_sender.send(action) {
                                    Ok(_) => {}
                                    Err(e) => tracing::error!(
                                        "Error sending action: {}",
                                        e
                                    ),
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error receiving event: {}", e)
                        }
                    }
                }
            });
        }

        for event_source in self.event_sources {
            let event_sender = event_sender.clone();
            tasks.spawn(async move {
                tracing::info!("Starting event source...");
                let mut event_stream = event_source
                    .get_event_stream()
                    .await
                    .expect("Event source didn't return event stream");
                while let Some(event) = event_stream.next().await {
                    match event_sender.send(event) {
                        Ok(_) => {}
                        Err(e) => tracing::error!("Error sending event: {}", e),
                    }
                }
            });
        }

        Ok(tasks)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use async_trait::async_trait;
    use futures::stream;
    use tokio::time::sleep;

    use super::*;
    use crate::types::{Action, Event, EventStream};

    struct MockEventSource {
        events: Vec<Event>,
    }

    #[async_trait]
    impl EventSource<Event> for MockEventSource {
        async fn get_event_stream(
            &self,
        ) -> Result<EventStream<'_, Event>, KazukaError> {
            let stream = stream::iter(self.events.clone());
            Ok(Box::pin(stream))
        }
    }

    struct MockStrategy {
        events: Arc<Mutex<Vec<Event>>>,
    }

    #[async_trait]
    impl Strategy<Event, Action> for MockStrategy {
        async fn process_event(&mut self, event: Event) -> Vec<Action> {
            self.events.lock().unwrap().push(event.clone());
            match event {
                Event::Transaction => vec![Action::SubmitTxToMempool],
                _ => vec![],
            }
        }
    }

    struct MockExecutor {
        actions: Arc<Mutex<Vec<Action>>>,
    }

    #[async_trait]
    impl Executor<Action> for MockExecutor {
        async fn execute(&self, action: Action) -> Result<(), KazukaError> {
            self.actions.lock().unwrap().push(action);
            Ok(())
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_engine_pipeline() {
        let incoming_events = vec![Event::NewBlock, Event::Transaction];
        let received_events = Arc::new(Mutex::new(vec![]));
        let produced_actions = Arc::new(Mutex::new(vec![]));

        let strategy = MockStrategy {
            events: Arc::clone(&received_events),
        };
        let executor = MockExecutor {
            actions: produced_actions.clone(),
        };
        let engine = Engine::new()
            .add_event_source(Box::new(MockEventSource {
                events: incoming_events.clone(),
            }))
            .add_strategy(Box::new(strategy))
            .add_executor(Box::new(executor));

        let mut tasks = engine.run().await.expect("Engine failed to run");

        sleep(Duration::from_millis(200)).await;

        tasks.shutdown().await;

        let received_events = received_events.lock().unwrap().clone();
        assert_eq!(received_events, incoming_events);

        let produced_actions = produced_actions.lock().unwrap().clone();
        assert_eq!(produced_actions.len(), 1);
        assert_eq!(
            produced_actions[0],
            Action::SubmitTxToMempool
        );
    }
}
