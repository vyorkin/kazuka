use http::HeaderName;
use http_body_util::Full;
use hyper::body::Bytes;
use jsonrpsee::{
    core::middleware::{
        Batch, BatchEntry, BatchEntryErr, Notification, RpcServiceT,
    },
    types::Request,
};
use tower::Layer;

// To authenticate your request, Flashbots endpoints require you to
// sign the payload and include the signed payload in the X-Flashbots-Signature
// header of your request.
// See: https://docs.flashbots.net/flashbots-protect/nonce-management#authentication

static FLASHBOTS_HEADER: HeaderName =
    HeaderName::from_static("x-flashbots-signature");

type HttpRequest = http::Request<Full<Bytes>>;

/// Layer that adds a request header with a signed payload.
pub struct AuthLayer<SI> {
    pub request: HttpRequest,
    pub signer: SI,
}

impl<SI> AuthLayer<SI> {
    pub fn new(signer: SI, request: HttpRequest) -> Self {
        Self { signer, request }
    }
}

impl<S, SI: Clone> Layer<S> for AuthLayer<SI> {
    type Service = Auth<S, SI>;

    fn layer(&self, service: S) -> Self::Service {
        Auth {
            service,
            signer: self.signer.clone(),
            request: self.request.clone(),
        }
    }
}

/// Middleware that signs the request body and
/// adds the signature to the x-flashbots-signature header.
/// For more info, see
/// <https://docs.flashbots.net/flashbots-auction/advanced/rpc-endpoint#authentication>.
pub struct Auth<S, SI> {
    service: S,
    signer: SI,
    request: HttpRequest,
}

impl<S, SI> Auth<S, SI> {
    fn sign<'a>(&self, request: Request<'a>) -> Request<'a> {
        use http_body_util::BodyExt;

        let (mut parts, body) = self.request.clone().into_parts();
        if parts.method != http::Method::POST {
            return request;
        }
        request
    }
}

// !
// We can not add X-Flashbots-Signature header here.
// So there is no point in implementing RpcServiceT.

impl<S, SI> RpcServiceT for Auth<S, SI>
where
    S: RpcServiceT + Send + Sync + Clone + 'static,
{
    type MethodResponse = S::MethodResponse;
    type NotificationResponse = S::NotificationResponse;
    type BatchResponse = S::BatchResponse;

    fn batch<'a>(
        &self,
        batch: Batch<'a>,
    ) -> impl Future<Output = Self::BatchResponse> + Send + 'a {
        let entries = batch
            .into_iter()
            .map(|entry| match entry {
                Ok(BatchEntry::Call(req)) => {
                    Ok(BatchEntry::Call(self.sign(req)))
                }
                Ok(BatchEntry::Notification(n)) => {
                    Ok(BatchEntry::Notification(n))
                }
                Err(err) => Err(err),
            })
            .collect::<Vec<_>>();
        self.service.batch(Batch::from(entries))
    }

    fn call<'a>(
        &self,
        request: Request<'a>,
    ) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
        let request = self.sign(request);
        self.service.call(request)
    }

    fn notification<'a>(
        &self,
        n: Notification<'a>,
    ) -> impl Future<Output = Self::NotificationResponse> + Send + 'a {
        self.service.notification(n)
    }
}
