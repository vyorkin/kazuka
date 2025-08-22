use std::task::{Context, Poll};

use alloy::{
    primitives::{B256, keccak256},
    transports::BoxFuture,
};
use futures_util::FutureExt;
use http::{HeaderName, HeaderValue, Request};
use http_body_util::Full;
use hyper::body::Bytes;
use jsonrpsee::http_client::{
    HttpBody, HttpRequest, transport::Error as TransportError,
};
use tower::{Layer, Service};

// To authenticate your request, Flashbots endpoints require you to
// sign the payload and include the signed payload in the X-Flashbots-Signature
// header of your request.
// See: https://docs.flashbots.net/flashbots-protect/nonce-management#authentication

static FLASHBOTS_HEADER: HeaderName =
    HeaderName::from_static("x-flashbots-signature");

#[derive(Clone)]
pub struct AuthService<Service, Signer> {
    service: Service,
    signer: Signer,
}

impl<S, Signer> Service<HttpRequest> for AuthService<S, Signer>
where
    Signer: alloy::signers::Signer + Clone + Send + Sync + 'static,
    S: Service<HttpRequest> + Clone + Send + 'static,
    S::Future: Send,
    S::Error: Into<TransportError>,
    // S::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = TransportError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        use http_body_util::BodyExt;

        let service_clone = self.service.clone();
        // Even though the original service is ready, the clone might not be.
        // See: https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        // Here is how we take the service that is ready.
        let mut service = std::mem::replace(&mut self.service, service_clone);

        // If method is not POST, return an error.
        let (mut parts, body) = request.into_parts();

        let is_json = parts
            .headers
            .get(http::header::CONTENT_TYPE)
            .map(|v| v == HeaderValue::from_static("application/json"))
            .unwrap_or(false);

        let has_flashbots_header =
            parts.headers.contains_key(FLASHBOTS_HEADER.clone());

        // If content-type is not JSON,
        // or flashbots header already exists, just pass through the request.
        if !is_json
            || has_flashbots_header
            || parts.method != http::Method::POST
        {
            return async move {
                let request = Request::from_parts(parts, body);
                service.call(request).await.map_err(|e| e.into())
            }
            .boxed();
        }

        let signer = self.signer.clone();

        async move {
            let body_bytes: Bytes = body
                .collect()
                .await
                .expect("Failed to collect body")
                .to_bytes();

            let message = format!(
                "0x{:x}",
                B256::from(keccak256(body_bytes.as_ref()))
            );
            let message_bytes = message.into_bytes();
            let signature = signer
                .sign_message(&message_bytes)
                .await
                .expect("Failed to sign message");
            let header_str = format!("{:?}:0x{}", signer.address(), signature);
            let header_val = HeaderValue::from_str(&header_str)
                .expect("Flashbots header contains invalid characters");

            parts.headers.insert(FLASHBOTS_HEADER.clone(), header_val);

            let body = HttpBody::new(Full::new(body_bytes));

            let authenticated_request = HttpRequest::from_parts(parts, body);
            service
                .call(authenticated_request)
                .await
                .map_err(Into::into)
        }
        .boxed()
    }
}

/// Layer that applies [`AuthService`]
/// which adds a request header with a signed payload.
#[derive(Clone, Default)]
pub struct AuthLayer<Signer> {
    signer: Signer,
}

impl<Signer> AuthLayer<Signer> {
    pub fn new(signer: Signer) -> Self {
        Self { signer }
    }
}

impl<Signer: Clone, S> Layer<S> for AuthLayer<Signer> {
    type Service = AuthService<S, Signer>;

    fn layer(&self, service: S) -> Self::Service {
        AuthService {
            service,
            signer: self.signer.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::signers::local::PrivateKeySigner;
    use tower::service_fn;

    use super::*;

    #[tokio::test]
    async fn test_auth_service_adds_header_for_post_request() {
        let service = service_fn(|request: HttpRequest| async move {
            assert!(request.headers().contains_key(FLASHBOTS_HEADER.clone()));
            Ok::<_, TransportError>(())
        });

        let signer = PrivateKeySigner::random();
        let mut auth_service = AuthService { service, signer };

        let request = Request::builder()
            .method(http::Method::POST)
            .header("content-type", "application/json")
            .body(HttpBody::new(Full::new(
                Bytes::from_static(b"{\"key\":\"value\"}"),
            )))
            .unwrap();

        auth_service.call(HttpRequest::from(request)).await.unwrap();
    }

    #[tokio::test]
    async fn test_auth_service_passes_through_non_post_request() {
        let service = service_fn(|request: HttpRequest| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(!request.headers().contains_key(FLASHBOTS_HEADER.clone()));
            Ok::<_, TransportError>(())
        });

        let signer = PrivateKeySigner::random();
        let mut auth_service = AuthService { service, signer };

        let request = Request::builder()
            .method(http::Method::GET)
            .header("content-type", "application/json")
            .body(HttpBody::new(Full::new(
                Bytes::from_static(b"{\"key\":\"value\"}"),
            )))
            .unwrap();

        auth_service.call(HttpRequest::from(request)).await.unwrap();
    }

    #[tokio::test]
    async fn test_auth_service_passes_through_non_json_request() {
        let service = service_fn(|request: HttpRequest| async move {
            assert_eq!(
                request.headers().get("content-type").unwrap(),
                "text/plain"
            );
            assert!(!request.headers().contains_key(FLASHBOTS_HEADER.clone()));
            Ok::<_, TransportError>(())
        });

        let signer = PrivateKeySigner::random();
        let mut auth_service = AuthService { service, signer };

        let request = Request::builder()
            .method(http::Method::POST)
            .header("content-type", "text/plain")
            .body(HttpBody::new(Full::new(
                Bytes::from_static(b"whatever"),
            )))
            .unwrap();

        auth_service.call(HttpRequest::from(request)).await.unwrap();
    }

    #[tokio::test]
    async fn test_auth_service_passes_through_existing_header() {
        let service = service_fn(|request: HttpRequest| async move {
            assert!(request.headers().contains_key(FLASHBOTS_HEADER.clone()));
            Ok::<_, TransportError>(())
        });

        let signer = PrivateKeySigner::random();
        let mut auth_service = AuthService { service, signer };

        let request = Request::builder()
            .method(http::Method::POST)
            .header("content-type", "application/json")
            .header(FLASHBOTS_HEADER.clone(), "signature")
            .body(HttpBody::new(Full::new(
                Bytes::from_static(b"{\"key\":\"value\"}"),
            )))
            .unwrap();

        auth_service.call(HttpRequest::from(request)).await.unwrap();
    }
}
