use alloy::{rpc::types::mev::MevSendBundle, signers::Signer};
use async_trait::async_trait;
use jsonrpsee::http_client::HttpClientBuilder;
use kazuka_core::{error::KazukaError, types::Executor};
use kazuka_mev_share::rpc::{MevApiClient, middleware::AuthLayer};
use tower::ServiceBuilder;

/// An executor that sends bundles to the MEV-share matchmaker.
pub struct MevShareExecutor {
    mev_share_client: Box<dyn MevApiClient + Send + Sync>,
    /// Whether to actually submit bundles or just log them.
    dry_run: bool,
}

impl MevShareExecutor {
    pub fn new(
        url: String,
        dry_run: bool,
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
            dry_run,
        }
    }
}

#[async_trait]
impl Executor<MevSendBundle> for MevShareExecutor {
    async fn execute(&self, action: MevSendBundle) -> Result<(), KazukaError> {
        if self.dry_run {
            tracing::info!(
                "Submitting bundle [DRY RUN]: {:?}",
                action
            );
            return Ok(());
        } else {
            tracing::info!("Submitting bundle: {:?}", action);
        }

        let body = self.mev_share_client.send_bundle(action).await;
        match body {
            Ok(body) => tracing::info!("Bundle response: {:?}", body),
            Err(err) => tracing::error!("Bundle error: {:?}", err),
        };

        Ok(())
    }
}
