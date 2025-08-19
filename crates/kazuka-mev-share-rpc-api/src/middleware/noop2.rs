use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{FutureExt, future::BoxFuture};
use http::Response;
use hyper::body::Incoming;
use jsonrpsee::{
    http_client::{HttpBody, HttpRequest, HttpResponse},
    proc_macros::rpc,
    server::http::response::error_response,
    types::{ErrorCode, ErrorObject, ErrorObjectOwned},
};
use tower::{BoxError, Layer, Service};

// Service - this is the actual middleware, which does the work
// Layer - a wrapper for Service,
//         may contain state/params which are passed to the Service

#[derive(Clone)]
pub struct NoOpService<S> {
    service: S,
    called: Arc<Mutex<bool>>,
}

impl<S, B> Service<HttpRequest<B>> for NoOpService<S>
where
    S: Service<HttpRequest<B>, Response = HttpResponse>
        + Clone
        + Send
        + 'static,
    S::Response: 'static,
    S::Error: Into<BoxError> + 'static,
    S::Future: Send + 'static,
    B: http_body::Body<Data = Bytes> + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    // type Future = Pin<
    //     Box<
    //         dyn Future<Output = Result<Self::Response, Self::Error>>
    //             + Send
    //             + 'static,
    //     >,
    // >;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: HttpRequest<B>) -> Self::Future {
        use http_body_util::BodyExt;

        // return async move { Err("pzdc".to_string().into()) }.boxed();
        let (mut parts, body) = request.into_parts();
        if parts.method != http::Method::POST {
            return async move { Ok(internal_error()) }.boxed();
        }

        let service_clone = self.service.clone();
        // Even though the original service is ready, the clone might not be.
        // See: https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        // Here is how we take the service that is ready.
        let mut service = std::mem::replace(&mut self.service, service_clone);

        let mut called = self.called.lock().unwrap();
        *called = true;
        drop(called);

        async move {
            let request = HttpRequest::from_parts(parts, body);
            service.call(request).await.map_err(Into::into)
        }
        .boxed()
    }
}

fn internal_error() -> HttpResponse {
    #[derive(serde::Deserialize)]
    struct ErrorResponse<'a> {
        #[serde(borrow)]
        error: ErrorObject<'a>,
    }

    let error = serde_json::from_str::<ErrorResponse>("pdzc")
        .map(|payload| payload.error)
        .unwrap_or_else(|_| ErrorObject::from(ErrorCode::InternalError));

    error_response(error)
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

#[rpc(server, client)]
pub trait RpcApi {
    #[method(name = "say_hello")]
    async fn say_hello(&self) -> Result<String, ErrorObjectOwned>;
}

struct RpcServer;

#[async_trait]
impl RpcApiServer for RpcServer {
    async fn say_hello(&self) -> Result<String, ErrorObjectOwned> {
        Ok("hello".to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use jsonrpsee::{http_client::HttpClientBuilder, server::Server};
    use tower::ServiceBuilder;

    use super::*;

    async fn run_server() -> anyhow::Result<SocketAddr> {
        let server = Server::builder().build("127.0.0.1:3000").await?;
        let addr = server.local_addr()?;
        let handle = server.start(RpcServer.into_rpc());
        tokio::spawn(handle.stopped());
        Ok(addr)
    }

    #[tokio::test]
    async fn test_noop2_layer() -> anyhow::Result<()> {
        let server_addr = run_server().await?;

        let noop_layer = NoOpLayer::new();
        let middleware = ServiceBuilder::new().layer(noop_layer.clone());

        let client = HttpClientBuilder::default()
            // .set_http_middleware(middleware)
            .build(format!("http://{server_addr}"))?;

        assert!(!noop_layer.was_called());

        let reply: String = client.say_hello().await?;

        assert_eq!(reply, "hello");

        assert!(noop_layer.was_called());

        Ok(())
    }
}
