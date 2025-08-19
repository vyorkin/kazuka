use std::{
    error::Error,
    task::{Context, Poll},
};

use alloy::{
    primitives::{B256, keccak256},
    signers::Signer,
    transports::BoxFuture,
};
use http::{HeaderName, HeaderValue, Request};
use http_body_util::Full;
use hyper::body::Bytes;
use tower::{BoxError, Layer, Service};

// To authenticate your request, Flashbots endpoints require you to
// sign the payload and include the signed payload in the X-Flashbots-Signature
// header of your request.
// See: https://docs.flashbots.net/flashbots-protect/nonce-management#authentication

static FLASHBOTS_HEADER: HeaderName =
    HeaderName::from_static("x-flashbots-signature");

/// Layer that applies [`FlashbotsSigner`]
/// which adds a request header with a signed payload.
pub struct FlashbotsSignerLayer<S> {
    signer: S,
}

impl<S> FlashbotsSignerLayer<S> {
    pub fn new(signer: S) -> Self {
        Self { signer }
    }
}

impl<S: Clone, I> Layer<I> for FlashbotsSignerLayer<S> {
    type Service = FlashbotsSignerService<S, I>;

    fn layer(&self, inner: I) -> Self::Service {
        FlashbotsSignerService {
            signer: self.signer.clone(),
            inner,
        }
    }
}

/// Middleware that signs the request body and
/// adds the signature to the x-flashbots-signature header.
/// For more info, see
/// <https://docs.flashbots.net/flashbots-auction/advanced/rpc-endpoint#authentication>.
pub struct FlashbotsSignerService<S, I> {
    signer: S,
    inner: I,
}

type Req = Request<Full<Bytes>>;

impl<S, I> Service<Req> for FlashbotsSignerService<S, I>
where
    S: Signer + Clone + Send + Sync + 'static,
    I: Service<Req> + Clone + Send + 'static,
    I::Future: Send,
    I::Error: Into<BoxError> + 'static,
{
    type Response = I::Response;
    type Error = Box<dyn Error + Send + Sync>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: Req) -> Self::Future {
        use http_body_util::BodyExt;

        let inner_clone = self.inner.clone();
        // Even though the original service is ready, the clone might not be.
        // See: https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        // Here is how we take the service that is ready.
        let mut inner = std::mem::replace(&mut self.inner, inner_clone);

        // If method is not POST, return an error.
        let (mut parts, body) = request.into_parts();
        if parts.method != http::Method::POST {
            return Box::pin(async move {
                Err(format!(
                    "Invalid method: {}",
                    parts.method.as_str()
                )
                .into())
            });
        }

        let is_json = parts
            .headers
            .get(http::header::CONTENT_TYPE)
            .map(|v| v == HeaderValue::from_static("application/json"))
            .unwrap_or(false);

        let has_flashbots_header =
            parts.headers.contains_key(FLASHBOTS_HEADER.clone());

        // If content-type is not JSON,
        // or flashbots header already exists, just pass through the request.
        if !is_json || has_flashbots_header {
            return Box::pin(async move {
                let request = Request::from_parts(parts, body);
                inner.call(request).await.map_err(|e| e.into())
            });
        }

        let signer = self.signer.clone();

        Box::pin(async move {
            let body_bytes = body
                .collect()
                .await
                .map_err(|e| format!("Failed to collect body: {e}"))?
                .to_bytes();

            let message = format!(
                "0x{:x}",
                B256::from(keccak256(body_bytes.as_ref()))
            );
            let message_bytes = message.into_bytes();
            let signature = signer.sign_message(&message_bytes).await?;
            let header_str = format!("{:?}:0x{}", signer.address(), signature);
            let header_val = HeaderValue::from_str(&header_str)
                .expect("Flashbots header contains invalid characters");

            parts.headers.insert(FLASHBOTS_HEADER.clone(), header_val);

            let request = Request::from_parts(parts, Full::new(body_bytes));
            inner.call(request).await.map_err(Into::into)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use alloy::signers::local::PrivateKeySigner;
    use futures_util::future::{self, Ready};
    use http_body_util::BodyExt;

    use super::*;

    #[derive(Clone)]
    struct MockService;

    impl Service<Request<Full<Bytes>>> for MockService {
        type Response = Request<Full<Bytes>>;
        type Error = Infallible;
        type Future = Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: Request<Full<Bytes>>) -> Self::Future {
            future::ready(Ok(req))
        }
    }

    fn make_get_request() -> Request<Full<Bytes>> {
        Request::builder()
            .method(http::Method::GET)
            .uri("https://example.com")
            .body(Full::new(Bytes::new()))
            .unwrap()
    }

    fn make_post_request(
        body: Vec<u8>,
        content_type: Option<&str>,
        include_flashbots_header: bool,
    ) -> Request<Full<Bytes>> {
        let mut builder = Request::builder()
            .uri("https://example.com")
            .method(http::Method::POST);

        if let Some(ct) = content_type {
            builder = builder.header(http::header::CONTENT_TYPE, ct);
        }
        if include_flashbots_header {
            builder = builder.header(
                FLASHBOTS_HEADER.clone(),
                HeaderValue::from_static("already-signed"),
            );
        }
        builder.body(Full::new(Bytes::from(body.clone()))).unwrap()
    }

    fn make_flashbots_signer_service() -> (
        PrivateKeySigner,
        FlashbotsSignerService<PrivateKeySigner, MockService>,
    ) {
        let signer = PrivateKeySigner::random();
        let service = FlashbotsSignerService {
            signer: signer.clone(),
            inner: MockService,
        };
        (signer, service)
    }

    #[tokio::test]
    async fn test_flashbots_signature_header() {
        let (flashbots_signer, mut flashbots_service) =
            make_flashbots_signer_service();

        let request_bytes = vec![1u8; 32];
        let request = make_post_request(
            request_bytes.clone(),
            Some("application/json"),
            false,
        );
        let response = flashbots_service
            .call(request)
            .await
            .expect("should succeed");

        assert!(response.headers().contains_key(FLASHBOTS_HEADER.clone()));

        let header = response
            .headers()
            .get(FLASHBOTS_HEADER.clone())
            .unwrap()
            .to_str()
            .unwrap()
            .split(":0x")
            .collect::<Vec<_>>();

        let header_address = header[0];
        let header_signature = header[1];

        let expected_message = format!(
            "0x{:x}",
            B256::from(keccak256(request_bytes.clone()))
        );
        let expected_message_bytes = expected_message.into_bytes();
        let expected_signature = flashbots_signer
            .sign_message(&expected_message_bytes)
            .await
            .unwrap()
            .to_string();

        let signer_address = format!("{:?}", flashbots_signer.address());

        assert_eq!(header_address, signer_address);
        assert_eq!(header_signature, expected_signature);
    }

    #[tokio::test]
    async fn test_passes_through_non_json() {
        let (_, mut flashbots_service) = make_flashbots_signer_service();
        let request_bytes = vec![1u8; 16];
        let request = make_post_request(
            request_bytes.clone(),
            Some("text/plain"),
            false,
        );
        let response = flashbots_service
            .call(request)
            .await
            .expect("should pass through");
        let response_bytes =
            response.body().clone().collect().await.unwrap().to_bytes();

        assert_eq!(response_bytes, request_bytes);
        assert!(!response.headers().contains_key(FLASHBOTS_HEADER.clone()));
    }

    #[tokio::test]
    async fn test_passes_through_if_flashbots_header_present() {
        let (_, mut flashbots_service) = make_flashbots_signer_service();

        let request_bytes = vec![1u8; 32];

        let include_flashbots_header = true;
        let request = make_post_request(
            request_bytes.clone(),
            Some("application/json"),
            include_flashbots_header,
        );

        let response = flashbots_service
            .call(request)
            .await
            .expect("should pass through");

        // Flashbots header should be preserved
        assert!(response.headers().contains_key(FLASHBOTS_HEADER.clone()));

        assert_eq!(
            response.headers().get(FLASHBOTS_HEADER.clone()).unwrap(),
            HeaderValue::from_static("already-signed")
        );
    }

    #[tokio::test]
    async fn test_non_post_method_fails() {
        let (_, mut flashbots_service) = make_flashbots_signer_service();
        let request = make_get_request();
        let result = flashbots_service.call(request).await;

        assert!(result.is_err());

        let error_message = format!("{}", result.unwrap_err());
        assert!(error_message.contains("Invalid method"));
    }
}
