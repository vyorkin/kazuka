#[cfg(feature = "client")]
use alloy::rpc::types::mev::{
    EthCallBundle, EthCallBundleTransactionResult, EthCancelBundle,
    EthCancelPrivateTransaction, EthSendBundle, EthSendPrivateTransaction,
};
use alloy::{
    self,
    primitives::{B256, Bytes},
};
use async_trait::async_trait;
use jsonrpsee::{core::ClientError, proc_macros::rpc};
#[cfg(feature = "client")]
use tracing::instrument;

use crate::types::BundleHash;

/// jsonrpsee generated code.
///
/// This hides the generated client trait which is
/// replaced by the `EthBundleApiClient` trait.
mod rpc {
    use alloy::rpc::types::mev::{
        EthCallBundle, EthCallBundleTransactionResult, EthCancelBundle,
        EthSendBundle,
    };
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
            request: EthSendBundle,
        ) -> RpcResult<BundleHash>;

        /// The `eth_callBundle` is used to simulate a bundle against a specific
        /// block, including simulating a bundle at the top of the next block.
        #[method(name = "callBundle")]
        async fn call_bundle(
            &self,
            request: EthCallBundle,
        ) -> RpcResult<EthCallBundleTransactionResult>;

        /// The `eth_cancelBundle` is used to prevent a submitted bundle from
        /// being included on-chain.
        ///
        /// See [bundle cancellations](https://docs.flashbots.net/flashbots-auction/searchers/advanced/bundle-cancellations) for more information.
        #[method(name = "cancelBundle")]
        async fn cancel_bundle(
            &self,
            request: EthCancelBundle,
        ) -> RpcResult<()>;

        /// The `eth_sendPrivateTransaction` is used to send a single
        /// transaction to Flashbots. Flashbots will attempt to build a
        /// block including the transaction for the next 25 blocks.
        ///
        /// See [Private Transactions](https://docs.flashbots.net/flashbots-protect/additional-documentation/eth-sendPrivateTransaction) for more info.
        #[method(name = "sendPrivateTransaction")]
        async fn send_private_transaction(
            &self,
            request: EthSendPrivateTransaction,
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
            request: EthCancelPrivateTransaction,
        ) -> RpcResult<bool>;
    }
}

// Re-export the rpc server trait.
#[cfg(feature = "server")]
pub use rpc::EthBundleApiServer;

/// An dyn-trait compatible (vtable compatible) version of the `EthBundleApi`
/// trait.
#[cfg(feature = "client")]
#[async_trait]
pub trait EthBundleApiClient {
    /// The `eth_sendBundle` is used to send your bundles to the builder.
    async fn send_bundle(
        &self,
        request: EthSendBundle,
    ) -> Result<BundleHash, ClientError>;

    /// The `eth_callBundle` is used to simulate a bundle against a specific
    /// block, including simulating a bundle at the top of the next block.
    async fn call_bundle(
        &self,
        request: EthCallBundle,
    ) -> Result<EthCallBundleTransactionResult, ClientError>;

    /// The `eth_cancelBundle` is used to prevent a submitted bundle from
    /// being included on-chain.
    ///
    /// See [bundle cancellations](https://docs.flashbots.net/flashbots-auction/searchers/advanced/bundle-cancellations) for more information.
    async fn cancel_bundle(
        &self,
        request: EthCancelBundle,
    ) -> Result<(), ClientError>;

    /// The `eth_sendPrivateTransaction` is used to send a single
    /// transaction to Flashbots. Flashbots will attempt to build a
    /// block including the transaction for the next 25 blocks.
    ///
    /// See [Private Transactions](https://docs.flashbots.net/flashbots-protect/additional-documentation/eth-sendPrivateTransaction) for more info.
    async fn send_private_transaction(
        &self,
        request: EthSendPrivateTransaction,
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
        request: EthCancelPrivateTransaction,
    ) -> Result<bool, ClientError>;
}

#[cfg(feature = "client")]
#[async_trait]
impl<T> EthBundleApiClient for T
where
    T: rpc::EthBundleApiClient + Sync,
{
    #[instrument(skip(self))]
    async fn send_bundle(
        &self,
        request: EthSendBundle,
    ) -> Result<BundleHash, ClientError> {
        rpc::EthBundleApiClient::send_bundle(self, request).await
    }

    #[instrument(skip(self))]
    async fn call_bundle(
        &self,
        request: EthCallBundle,
    ) -> Result<EthCallBundleTransactionResult, ClientError> {
        rpc::EthBundleApiClient::call_bundle(self, request).await
    }

    #[instrument(skip(self))]
    async fn cancel_bundle(
        &self,
        request: EthCancelBundle,
    ) -> Result<(), ClientError> {
        rpc::EthBundleApiClient::cancel_bundle(self, request).await
    }

    #[instrument(skip(self))]
    async fn send_private_transaction(
        &self,
        request: EthSendPrivateTransaction,
    ) -> Result<B256, ClientError> {
        rpc::EthBundleApiClient::send_private_transaction(self, request).await
    }

    #[instrument(skip(self))]
    async fn send_private_raw_transaction(
        &self,
        bytes: Bytes,
    ) -> Result<B256, ClientError> {
        rpc::EthBundleApiClient::send_private_raw_transaction(self, bytes).await
    }

    #[instrument(skip(self))]
    async fn cancel_private_transaction(
        &self,
        request: EthCancelPrivateTransaction,
    ) -> Result<bool, ClientError> {
        rpc::EthBundleApiClient::cancel_private_transaction(self, request).await
    }
}

#[cfg(all(test, feature = "client"))]
mod tests {
    use std::net::SocketAddr;

    use alloy::{
        primitives::{U256, address, b256, bytes},
        rpc::types::mev::{
            EthCallBundle, EthCallBundleTransactionResult, EthCancelBundle,
            EthSendBundle,
        },
    };
    use async_trait::async_trait;
    use jsonrpsee::{
        core::RpcResult, http_client::HttpClientBuilder, server::Server,
    };
    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use tracing_subscriber::{
        EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
    };

    use super::*;

    const DEFAULT_FILTER_LEVEL: &str = "trace";

    fn init_tracing() {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER_LEVEL));

        let _ = tracing_subscriber::registry()
            .with(fmt::layer())
            .with(env_filter)
            .try_init();
    }

    struct Client {
        inner: Box<dyn EthBundleApiClient>,
    }

    #[rpc(server, namespace = "eth")]
    #[async_trait]
    trait EthBundleApiMock {
        #[method(name = "sendBundle")]
        async fn send_bundle(
            &self,
            request: EthSendBundle,
        ) -> RpcResult<BundleHash>;

        #[method(name = "callBundle")]
        async fn call_bundle(
            &self,
            request: EthCallBundle,
        ) -> RpcResult<EthCallBundleTransactionResult>;

        #[method(name = "cancelBundle")]
        async fn cancel_bundle(
            &self,
            request: EthCancelBundle,
        ) -> RpcResult<()>;

        #[method(name = "sendPrivateTransaction")]
        async fn send_private_transaction(
            &self,
            request: EthSendPrivateTransaction,
        ) -> RpcResult<B256>;

        #[method(name = "sendPrivateRawTransaction")]
        async fn send_private_raw_transaction(
            &self,
            bytes: Bytes,
        ) -> RpcResult<B256>;

        #[method(name = "cancelPrivateTransaction")]
        async fn cancel_private_transaction(
            &self,
            request: EthCancelPrivateTransaction,
        ) -> RpcResult<bool>;
    }

    struct EthBundleApiMockServiceImpl;

    #[async_trait]
    impl EthBundleApiMockServer for EthBundleApiMockServiceImpl {
        async fn send_bundle(
            &self,
            _request: EthSendBundle,
        ) -> RpcResult<BundleHash> {
            Ok(BundleHash {
                bundle_hash: b256!(
                    "0xbeefbeefbeef0000000000000000000000000000000000000000000000000000"
                ),
            })
        }

        async fn call_bundle(
            &self,
            _request: EthCallBundle,
        ) -> RpcResult<EthCallBundleTransactionResult> {
            Ok(EthCallBundleTransactionResult {
                coinbase_diff: U256::from(10000000000063000u64),
                eth_sent_to_coinbase: U256::from(10000000000000000u64),
                from_address: address!(
                    "0x02A727155aeF8609c9f7F2179b2a1f560B39F5A0"
                ),
                gas_fees: U256::from(63000u64),
                gas_price: U256::from(476190476193u64),
                gas_used: 21000u64,
                to_address: Some(address!(
                    "0x73625f59CAdc5009Cb458B751b3E7b6b48C06f2C"
                )),
                tx_hash: b256!(
                    "0x669b4704a7d993a946cdd6e2f95233f308ce0c4649d2e04944e8299efcaa098a"
                ),
                value: Some(bytes!("0x")),
                revert: None,
            })
        }

        async fn cancel_bundle(
            &self,
            _request: EthCancelBundle,
        ) -> RpcResult<()> {
            Ok(())
        }

        async fn send_private_transaction(
            &self,
            _request: EthSendPrivateTransaction,
        ) -> RpcResult<B256> {
            Ok(b256!(
                "0x1111111111111111111111111111111111111111111111111111111111111111"
            ))
        }

        async fn send_private_raw_transaction(
            &self,
            _bytes: Bytes,
        ) -> RpcResult<B256> {
            Ok(b256!(
                "0x2222222222222222222222222222222222222222222222222222222222222222"
            ))
        }

        async fn cancel_private_transaction(
            &self,
            _request: EthCancelPrivateTransaction,
        ) -> RpcResult<bool> {
            Ok(true)
        }
    }

    async fn start_mock_server() -> anyhow::Result<SocketAddr> {
        let server = Server::builder().build("127.0.0.1:3001").await?;
        let addr = server.local_addr()?;

        let handle = server.start(EthBundleApiMockServiceImpl.into_rpc());
        tokio::spawn(handle.stopped());

        Ok(addr)
    }

    #[tokio::test]
    async fn test_send_bundle() -> anyhow::Result<()> {
        init_tracing();

        let server_addr = start_mock_server().await?;

        let client = HttpClientBuilder::default()
            .build(format!("http://{server_addr}"))?;

        let client = Client {
            inner: Box::new(client),
        };

        let request = EthSendBundle {
            txs: vec![bytes!(
                "0x02f86b0180843b9aca00852ecc889a0082520894c87037874aed04e51c29f582394217a0a2b89d808080c080a0a463985c616dd8ee17d7ef9112af4e6e06a27b071525b42182fe7b0b5c8b4925a00af5ca177ffef2ff28449292505d41be578bebb77110dfc09361d2fb56998260"
            )],
            block_number: 0x1,
            min_timestamp: None,
            max_timestamp: None,
            reverting_tx_hashes: vec![
                b256!(
                    "0x669b4704a7d993a946cdd6e2f95233f308ce0c4649d2e04944e8299efcaa098a"
                ),
                b256!(
                    "0xa839ee83465657cac01adc1d50d96c1b586ed498120a84a64749c0034b4f19fa"
                ),
            ],
            ..Default::default()
        };

        let response = client.inner.send_bundle(request).await;
        assert!(response.is_ok());
        let response = response.unwrap();

        assert_eq!(
            response,
            BundleHash {
                bundle_hash: b256!(
                    "0xbeefbeefbeef0000000000000000000000000000000000000000000000000000"
                ),
            }
        );

        Ok(())
    }
}
