use alloy::signers::Signer;
use async_trait::async_trait;
use jsonrpsee::http_client::HttpClientBuilder;
use kazuka_core::{error::KazukaError, types::Executor};

pub struct MevShareExecutor {
    // mev_share_client: Box<dyn MevApiClient + Send + Sync>,
    // provider: Arc<DynProvider<AnyNetwork>>,
}

impl MevShareExecutor {
    pub fn new(signer: impl Signer + Clone + 'static) -> Self {
        let http = HttpClientBuilder::default()
            .build("https://relay.flashbots.net:443")
            .expect("failed to build HTTP client");

        Self {
            // mev_share_client: Box::new(http),
        }
    }
}
//
// #[async_trait]
// impl Executor<SendBundleRequest> for MevShareExecutor {
//     async fn execute(
//         &self,
//         action: SendBundleRequest,
//     ) -> Result<(), KazukaError> {
//         todo!()
//     }
// }
