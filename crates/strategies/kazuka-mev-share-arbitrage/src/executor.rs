use alloy::{rpc::types::mev::SendBundleRequest, signers::Signer};
use async_trait::async_trait;
use jsonrpsee::http_client::HttpClientBuilder;
use kazuka_core::{error::KazukaError, types::Executor};
use kazuka_mev_share::rpc::{MevApiClient, middleware::AuthLayer};
use tower::ServiceBuilder;

pub struct MevShareExecutor {
    mev_share_client: Box<dyn MevApiClient + Send + Sync>,
    // provider: Arc<DynProvider<AnyNetwork>>,
}

impl MevShareExecutor {
    pub fn new(
        url: String,
        signer: impl Signer + Clone + Send + Sync + 'static,
    ) -> Self {
        let http_middleware =
            ServiceBuilder::new().layer(AuthLayer::new(signer));

        let client = HttpClientBuilder::default()
            .set_http_middleware(http_middleware)
            .build(url)
            .expect("Failed to build HTTP client");

        Self {
            mev_share_client: Box::new(client),
        }
    }
}
//
#[async_trait]
impl Executor<SendBundleRequest> for MevShareExecutor {
    async fn execute(
        &self,
        action: SendBundleRequest,
    ) -> Result<(), KazukaError> {
        todo!()
    }
}
