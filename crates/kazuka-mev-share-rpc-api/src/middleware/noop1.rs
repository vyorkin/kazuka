use std::{
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use tower::{Layer, Service};

// Service - this is the actual middleware, which does the work
// Layer - a wrapper for Service,
//         may contain state/params which are passed to the Service

#[derive(Clone)]
pub struct NoOpService<S> {
    service: S,
    called: Arc<Mutex<bool>>,
}

impl<S, Request> Service<Request> for NoOpService<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut called = self.called.lock().unwrap();
        *called = true;
        drop(called);

        self.service.call(req)
    }
}

#[derive(Clone, Default)]
pub struct NoOpLayer {
    called: Arc<Mutex<bool>>,
}

impl NoOpLayer {
    pub fn new() -> Self {
        Self {
            called: Arc::new(Mutex::new(false)),
        }
    }

    pub fn was_called(&self) -> bool {
        *self.called.lock().unwrap()
    }
}

impl<S> Layer<S> for NoOpLayer {
    type Service = NoOpService<S>;

    fn layer(&self, service: S) -> Self::Service {
        NoOpService {
            service,
            called: self.called.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        future::{self, Ready},
        task::{Context, Poll},
    };

    use tower::Service;

    use super::*;

    #[derive(Clone)]
    struct EchoService;

    impl Service<&'static str> for EchoService {
        type Response = &'static str;
        type Error = ();
        type Future = Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: &'static str) -> Self::Future {
            future::ready(Ok(req))
        }
    }

    #[tokio::test]
    async fn test_noop1_layer() {
        let noop_layer = NoOpLayer::new();
        let echo_service = EchoService;
        let mut svc = noop_layer.layer(echo_service);

        assert!(!noop_layer.was_called());

        let response = svc.call("foo").await.unwrap();
        assert_eq!(response, "foo");

        assert!(noop_layer.was_called());
    }
}
