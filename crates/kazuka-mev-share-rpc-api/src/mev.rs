use alloy::rpc::types::mev::{
    SendBundleRequest, SimBundleOverrides, SimBundleResponse,
};
use async_trait::async_trait;
use jsonrpsee::{core::ClientError, proc_macros::rpc};
use tracing::instrument;

use crate::types::SendBundleResponse;

/// jsonrpsee generated code.
///
/// This hides the generated client trait which is replaced by the
/// `MevApiClient` trait.
mod rpc {
    use jsonrpsee::core::RpcResult;

    use super::*;

    #[cfg_attr(not(feature = "server"), rpc(client, namespace = "mev"))]
    #[cfg_attr(not(feature = "client"), rpc(server, namespace = "mev"))]
    #[cfg_attr(
        all(feature = "client", feature = "server"),
        rpc(client, server, namespace = "mev")
    )]
    #[async_trait]
    pub trait MevApi {
        /// Submits bundle to the relay.
        /// Takes in a bundle and provides a bundle hash as a return value.
        #[method(name = "sendBundle")]
        async fn send_bundle(
            &self,
            request: SendBundleRequest,
        ) -> RpcResult<SendBundleResponse>;

        /// Similar to `mev_sendBundle` but instead of
        /// submitting a bundle to the relay, it returns a simulation result.
        /// Only fully matched bundles can be simulated.
        #[method(name = "simBundle")]
        async fn sim_bundle(
            &self,
            bundle: SendBundleRequest,
            sim_overrides: SimBundleOverrides,
        ) -> RpcResult<SimBundleResponse>;
    }
}

// Re-export the rpc server trait.
#[cfg(feature = "server")]
pub use rpc::MevApiServer;

/// An dyn-trait compatible (vtable compatible) version of the `MevApiClient`
/// trait.
///
/// Basically this trait allows doing this:
/// `let client = Box::new(client) as Box<dyn MevApiClient>`;
#[cfg(feature = "client")]
#[async_trait]
pub trait MevApiClient {
    /// Submitting bundles to the relay. It takes in a bundle and provides a
    /// bundle hash as a return value.
    async fn send_bundle(
        &self,
        request: SendBundleRequest,
    ) -> Result<SendBundleResponse, ClientError>;

    /// Similar to `mev_sendBundle` but instead of submitting a bundle to the
    /// relay, it returns a simulation result. Only fully matched bundles
    /// can be simulated.
    async fn sim_bundle(
        &self,
        bundle: SendBundleRequest,
        sim_overrides: SimBundleOverrides,
    ) -> Result<SimBundleResponse, ClientError>;
}

#[cfg(feature = "client")]
#[async_trait]
impl<T> MevApiClient for T
where
    T: rpc::MevApiClient + Sync,
{
    #[instrument(skip(self))]
    async fn send_bundle(
        &self,
        request: SendBundleRequest,
    ) -> Result<SendBundleResponse, ClientError> {
        rpc::MevApiClient::send_bundle(self, request).await
    }

    #[instrument(skip(self))]
    async fn sim_bundle(
        &self,
        bundle: SendBundleRequest,
        sim_overrides: SimBundleOverrides,
    ) -> Result<SimBundleResponse, ClientError> {
        rpc::MevApiClient::sim_bundle(self, bundle, sim_overrides).await
    }
}

#[cfg(all(test, feature = "client"))]
mod tests {
    use std::net::SocketAddr;

    use alloy::{
        primitives::{U256, b256},
        rpc::types::mev::{
            SendBundleRequest, SimBundleOverrides, SimBundleResponse,
        },
        signers::local::PrivateKeySigner,
    };
    use async_trait::async_trait;
    use jsonrpsee::{
        core::RpcResult, http_client::HttpClientBuilder, server::Server,
    };
    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use tower::ServiceBuilder;
    use tracing_subscriber::{
        EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
    };

    use super::*;
    use crate::middleware::AuthLayer;

    const DEFAULT_FILTER_LEVEL: &str = "trace";

    fn init_tracing() {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER_LEVEL));

        let _ = tracing_subscriber::registry()
            .with(fmt::layer())
            .with(env_filter)
            .try_init();
    }

    #[rpc(server, namespace = "mev")]
    #[async_trait]
    trait MevApiMock {
        #[method(name = "sendBundle")]
        async fn send_bundle(
            &self,
            request: SendBundleRequest,
        ) -> RpcResult<SendBundleResponse>;

        #[method(name = "simBundle")]
        async fn sim_bundle(
            &self,
            bundle: SendBundleRequest,
            sim_overrides: SimBundleOverrides,
        ) -> RpcResult<SimBundleResponse>;
    }

    struct MevApiMockServerImpl;

    #[async_trait]
    impl MevApiMockServer for MevApiMockServerImpl {
        async fn send_bundle(
            &self,
            _request: SendBundleRequest,
        ) -> RpcResult<SendBundleResponse> {
            Ok(SendBundleResponse {
                bundle_hash: b256!(
                    "0x0000000000000000000000000000000000000000000000000000000000000000"
                ),
            })
        }

        async fn sim_bundle(
            &self,
            _bundle: SendBundleRequest,
            _sim_overrides: SimBundleOverrides,
        ) -> RpcResult<SimBundleResponse> {
            Ok(SimBundleResponse {
                success: true,
                error: None,
                state_block: 0x1,
                mev_gas_price: U256::from(476190476193u64),
                profit: U256::from(1000000),
                refundable_value: U256::from(5000),
                gas_used: 1000,
                logs: None,
                exec_error: None,
                revert: None,
            })
        }
    }

    async fn start_mock_server() -> anyhow::Result<SocketAddr> {
        let server = Server::builder().build("127.0.0.1:3000").await?;
        let addr = server.local_addr()?;

        let handle = server.start(MevApiMockServerImpl.into_rpc());
        tokio::spawn(handle.stopped());

        Ok(addr)
    }

    #[tokio::test]
    async fn test_send_bundle() -> anyhow::Result<()> {
        init_tracing();

        let server_addr = start_mock_server().await?;
        let signer = PrivateKeySigner::random();
        let http_middleware =
            ServiceBuilder::new().layer(AuthLayer::new(signer));

        let client = HttpClientBuilder::default()
            .set_http_middleware(http_middleware)
            .build(format!("http://{server_addr}"))?;
        let client = Box::new(client) as Box<dyn MevApiClient>;

        let request = SendBundleRequest {
            protocol_version: Default::default(),
            inclusion: Default::default(),
            bundle_body: vec![],
            validity: None,
            privacy: None,
        };
        let response = client.send_bundle(request).await;

        assert!(response.is_ok());
        let response = response.unwrap();

        let expected_bundle_hash = b256!(
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(
            response.bundle_hash,
            expected_bundle_hash
        );

        Ok(())
    }
}
