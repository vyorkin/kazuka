use alloy::primitives::{B256, Bytes};
use async_trait::async_trait;
use jsonrpsee::{core::ClientError, proc_macros::rpc};

use crate::types::*;

/// jsonrpsee generated code.
///
/// This hides the generated client trait which is
/// replaced by the `EthBundleApiClient` trait.
mod rpc {
    use jsonrpsee::core::RpcResult;

    use super::*;

    #[cfg_attr(not(feature = "server"), rpc(client, namespace = "eth"))]
    #[cfg_attr(not(feature = "client"), rpc(server, namespace = "eth"))]
    #[cfg_attr(
        all(feature = "client", feature = "server"),
        rpc(client, server, namespace = "eth")
    )]
    #[async_trait]
    pub trait EthBundleApi {
        /// The `eth_sendBundle` is used to send your bundles to the builder.
        #[method(name = "sendBundle")]
        async fn send_bundle(
            &self,
            request: eth::SendBundleRequest,
        ) -> RpcResult<eth::BundleHash>;

        /// The `eth_callBundle` is used to simulate a bundle against a specific
        /// block, including simulating a bundle at the top of the next block.
        #[method(name = "callBundle")]
        async fn call_bundle(
            &self,
            request: eth::CallBundleRequest,
        ) -> RpcResult<eth::CallBundleTransactionResult>;

        /// The `eth_cancelBundle` is used to prevent a submitted bundle from
        /// being included on-chain.
        ///
        /// See [bundle cancellations](https://docs.flashbots.net/flashbots-auction/searchers/advanced/bundle-cancellations) for more information.
        #[method(name = "cancelBundle")]
        async fn cancel_bundle(
            &self,
            request: eth::CancelBundleRequest,
        ) -> RpcResult<()>;

        /// The `eth_sendPrivateTransaction` is used to send a single
        /// transaction to Flashbots. Flashbots will attempt to build a
        /// block including the transaction for the next 25 blocks.
        ///
        /// See [Private Transactions](https://docs.flashbots.net/flashbots-protect/additional-documentation/eth-sendPrivateTransaction) for more info.
        #[method(name = "sendPrivateTransaction")]
        async fn send_private_transaction(
            &self,
            request: eth::PrivateTransactionRequest,
        ) -> RpcResult<B256>;

        /// The `eth_sendPrivateRawTransaction` method is used to send private
        /// transactions to the RPC endpoint. Private transactions are
        /// protected from frontrunning and kept private until included
        /// in a block. A request to this endpoint needs to follow
        /// the standard `eth_sendRawTransaction`.
        #[method(name = "sendPrivateRawTransaction")]
        async fn send_private_raw_transaction(
            &self,
            bytes: Bytes,
        ) -> RpcResult<B256>;

        /// The `eth_cancelPrivateTransaction` method stops private transactions
        /// from being submitted for future blocks.
        ///
        /// A transaction can only be cancelled if the request is signed by the
        /// same key as the `eth_sendPrivateTransaction` call submitting
        /// the transaction in first place.
        #[method(name = "cancelPrivateTransaction")]
        async fn cancel_private_transaction(
            &self,
            request: eth::CancelPrivateTransactionRequest,
        ) -> RpcResult<bool>;
    }
}

/// An dyn-trait compatible (vtable compatible) version of the `EthBundleApi`
/// trait.
#[cfg(feature = "client")]
#[async_trait]
pub trait EthBundleApiClient {
    /// The `eth_sendBundle` is used to send your bundles to the builder.
    async fn send_bundle(
        &self,
        request: eth::SendBundleRequest,
    ) -> Result<eth::BundleHash, ClientError>;

    /// The `eth_callBundle` is used to simulate a bundle against a specific
    /// block, including simulating a bundle at the top of the next block.
    async fn call_bundle(
        &self,
        request: eth::CallBundleRequest,
    ) -> Result<eth::CallBundleTransactionResult, ClientError>;

    /// The `eth_cancelBundle` is used to prevent a submitted bundle from
    /// being included on-chain.
    ///
    /// See [bundle cancellations](https://docs.flashbots.net/flashbots-auction/searchers/advanced/bundle-cancellations) for more information.
    async fn cancel_bundle(
        &self,
        request: eth::CancelBundleRequest,
    ) -> Result<(), ClientError>;

    /// The `eth_sendPrivateTransaction` is used to send a single
    /// transaction to Flashbots. Flashbots will attempt to build a
    /// block including the transaction for the next 25 blocks.
    ///
    /// See [Private Transactions](https://docs.flashbots.net/flashbots-protect/additional-documentation/eth-sendPrivateTransaction) for more info.
    async fn send_private_transaction(
        &self,
        request: eth::PrivateTransactionRequest,
    ) -> Result<B256, ClientError>;

    /// The `eth_sendPrivateRawTransaction` method is used to send private
    /// transactions to the RPC endpoint. Private transactions are
    /// protected from frontrunning and kept private until included
    /// in a block. A request to this endpoint needs to follow
    /// the standard `eth_sendRawTransaction`.
    async fn send_private_raw_transaction(
        &self,
        bytes: Bytes,
    ) -> Result<B256, ClientError>;

    /// The `eth_cancelPrivateTransaction` method stops private transactions
    /// from being submitted for future blocks.
    ///
    /// A transaction can only be cancelled if the request is signed by the
    /// same key as the `eth_sendPrivateTransaction` call submitting
    /// the transaction in first place.
    async fn cancel_private_transaction(
        &self,
        request: eth::CancelPrivateTransactionRequest,
    ) -> Result<bool, ClientError>;
}

#[cfg(feature = "client")]
#[async_trait]
impl<T> EthBundleApiClient for T
where
    T: rpc::EthBundleApiClient + Sync,
{
    async fn send_bundle(
        &self,
        request: eth::SendBundleRequest,
    ) -> Result<eth::BundleHash, ClientError> {
        rpc::EthBundleApiClient::send_bundle(self, request).await
    }

    async fn call_bundle(
        &self,
        request: eth::CallBundleRequest,
    ) -> Result<eth::CallBundleTransactionResult, ClientError> {
        rpc::EthBundleApiClient::call_bundle(self, request).await
    }

    async fn cancel_bundle(
        &self,
        request: eth::CancelBundleRequest,
    ) -> Result<(), ClientError> {
        rpc::EthBundleApiClient::cancel_bundle(self, request).await
    }

    async fn send_private_transaction(
        &self,
        request: eth::PrivateTransactionRequest,
    ) -> Result<B256, ClientError> {
        rpc::EthBundleApiClient::send_private_transaction(self, request).await
    }

    async fn send_private_raw_transaction(
        &self,
        bytes: Bytes,
    ) -> Result<B256, ClientError> {
        rpc::EthBundleApiClient::send_private_raw_transaction(self, bytes).await
    }

    async fn cancel_private_transaction(
        &self,
        request: eth::CancelPrivateTransactionRequest,
    ) -> Result<bool, ClientError> {
        rpc::EthBundleApiClient::cancel_private_transaction(self, request).await
    }
}

#[cfg(all(test, feature = "client"))]
mod tests {
    use std::net::SocketAddr;

    use alloy::{primitives::b256, signers::local::PrivateKeySigner};
    use async_trait::async_trait;
    use jsonrpsee::{
        core::RpcResult, http_client::HttpClientBuilder, server::Server,
    };
    use tower::ServiceBuilder;

    use super::*;
    use crate::middleware::auth::AuthLayer;

    struct Client {
        inner: Box<dyn EthBundleApiClient>,
    }

    #[rpc(server, namespace = "eth")]
    #[async_trait]
    trait EthBundleApiMock {
        #[method(name = "sendBundle")]
        async fn send_bundle(
            &self,
            request: eth::SendBundleRequest,
        ) -> RpcResult<eth::BundleHash>;

        #[method(name = "callBundle")]
        async fn call_bundle(
            &self,
            request: eth::CallBundleRequest,
        ) -> RpcResult<eth::CallBundleTransactionResult>;

        #[method(name = "cancelBundle")]
        async fn cancel_bundle(
            &self,
            request: eth::CancelBundleRequest,
        ) -> RpcResult<()>;

        #[method(name = "sendPrivateTransaction")]
        async fn send_private_transaction(
            &self,
            request: eth::PrivateTransactionRequest,
        ) -> RpcResult<B256>;

        #[method(name = "sendPrivateRawTransaction")]
        async fn send_private_raw_transaction(
            &self,
            bytes: Bytes,
        ) -> RpcResult<B256>;

        #[method(name = "cancelPrivateTransaction")]
        async fn cancel_private_transaction(
            &self,
            request: eth::CancelPrivateTransactionRequest,
        ) -> RpcResult<bool>;
    }

    struct EthBundleApiMockServiceImpl;

    #[async_trait]
    impl EthBundleApiMockServer for EthBundleApiMockServiceImpl {
        async fn send_bundle(
            &self,
            _request: eth::SendBundleRequest,
        ) -> RpcResult<eth::BundleHash> {
            Ok(eth::BundleHash {
                bundle_hash: b256!(
                    "0x0000000000000000000000000000000000000000000000000000000000000000"
                ),
            })
        }

        async fn call_bundle(
            &self,
            request: eth::CallBundleRequest,
        ) -> RpcResult<eth::CallBundleTransactionResult> {
            todo!()
        }

        async fn cancel_bundle(
            &self,
            request: eth::CancelBundleRequest,
        ) -> RpcResult<()> {
            todo!()
        }

        async fn send_private_transaction(
            &self,
            request: eth::PrivateTransactionRequest,
        ) -> RpcResult<B256> {
            todo!()
        }

        async fn send_private_raw_transaction(
            &self,
            bytes: Bytes,
        ) -> RpcResult<B256> {
            todo!()
        }

        async fn cancel_private_transaction(
            &self,
            request: eth::CancelPrivateTransactionRequest,
        ) -> RpcResult<bool> {
            todo!()
        }
    }

    async fn start_mock_server() -> anyhow::Result<SocketAddr> {
        let server = Server::builder().build("127.0.0.1:3000").await?;
        let addr = server.local_addr()?;

        let handle = server.start(EthBundleApiMockServiceImpl.into_rpc());
        tokio::spawn(handle.stopped());

        Ok(addr)
    }

    #[tokio::test]
    async fn test_send_bundle() -> anyhow::Result<()> {
        let server_addr = start_mock_server().await?;
        let signer = PrivateKeySigner::random();

        let client = HttpClientBuilder::default()
            .build(format!("http://{server_addr}"))?;

        let client = Client {
            inner: Box::new(client),
        };

        // client.inner.send_bundle(request)

        Ok(())
    }
}
