use jsonrpsee::{proc_macros::rpc, types::ErrorObjectOwned};

use crate::{SendBundleRequest, SendBundleResponse};

mod rpc {
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
        ) -> Result<SendBundleResponse, ErrorObjectOwned>;
    }
}

#[cfg(all(test, feature = "client"))]
mod tests {
    use std::{net::SocketAddr, str::FromStr};

    use alloy::{
        primitives::{B256, b256},
        signers::local::PrivateKeySigner,
    };
    use async_trait::async_trait;
    use jsonrpsee::{
        http_client::HttpClientBuilder,
        server::{Server, middleware::http::HostFilterLayer},
    };
    use tower::ServiceBuilder;

    use super::*;
    use crate::{
        mev::rpc::{MevApiClient, MevApiServer},
        middleware::auth3::AuthLayer,
    };

    struct MockMevApiServer;

    #[async_trait]
    impl MevApiServer for MockMevApiServer {
        async fn send_bundle(
            &self,
            _request: SendBundleRequest,
        ) -> Result<SendBundleResponse, ErrorObjectOwned> {
            let bundle_hash = b256!(
                "0xda7f09ac9b43acb4eb7d7c74dd5de20906ddd33fd4d82d8cb96997694b2d8e79"
            );
            Ok(SendBundleResponse { bundle_hash })
        }
    }

    async fn run_server() -> anyhow::Result<SocketAddr> {
        let server = Server::builder().build("127.0.0.1:3000").await?;
        let addr = server.local_addr()?;
        let handle = server.start(MockMevApiServer.into_rpc());
        tokio::spawn(handle.stopped());
        Ok(addr)
    }

    #[tokio::test]
    #[ignore = "todo"]
    async fn test_send_bundle_client() {
        let signer = PrivateKeySigner::random();

        let auth_layer = AuthLayer::new(signer);
        let auth_middleware = ServiceBuilder::new().layer(auth_layer);

        let host_filter_middleware = ServiceBuilder::new()
            .layer(HostFilterLayer::new(["example.com"]).unwrap());

        let http_client = HttpClientBuilder::default()
            .set_http_middleware(auth_middleware)
            // .set_http_middleware(host_filter_middleware)
            .build("http://localhost:3000")
            .unwrap();

        let result = MevApiClient::send_bundle(
            &http_client,
            SendBundleRequest {
                protocol_version: Default::default(),
                inclusion: Default::default(),
                bundle_body: vec![],
                validity: None,
                privacy: None,
            },
        )
        .await
        .unwrap();
    }
}
