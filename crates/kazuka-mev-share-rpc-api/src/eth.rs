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

    use alloy::primitives::{U64, U256, address, b256, bytes};
    use async_trait::async_trait;
    use jsonrpsee::{
        core::RpcResult, http_client::HttpClientBuilder, server::Server,
    };

    use super::*;
    use crate::types::eth::{BundleHash, CallBundleTransactionResult};

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
                    "0xbeefbeefbeef0000000000000000000000000000000000000000000000000000"
                ),
            })
        }

        async fn call_bundle(
            &self,
            _request: eth::CallBundleRequest,
        ) -> RpcResult<eth::CallBundleTransactionResult> {
            Ok(CallBundleTransactionResult {
                coinbase_diff: U256::from(10000000000063000u64),
                eth_sent_to_coinbase: U256::from(10000000000000000u64),
                from_address: address!(
                    "0x02A727155aeF8609c9f7F2179b2a1f560B39F5A0"
                ),
                gas_fees: U256::from(63000u64),
                gas_price: U256::from(476190476193u64),
                gas_used: 21000u64,
                to_address: address!(
                    "0x73625f59CAdc5009Cb458B751b3E7b6b48C06f2C"
                ),
                tx_hash: b256!(
                    "0x669b4704a7d993a946cdd6e2f95233f308ce0c4649d2e04944e8299efcaa098a"
                ),
                value: bytes!("0x"),
            })
        }

        async fn cancel_bundle(
            &self,
            _request: eth::CancelBundleRequest,
        ) -> RpcResult<()> {
            Ok(())
        }

        async fn send_private_transaction(
            &self,
            _request: eth::PrivateTransactionRequest,
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
            _request: eth::CancelPrivateTransactionRequest,
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
        let server_addr = start_mock_server().await?;

        let client = HttpClientBuilder::default()
            .build(format!("http://{server_addr}"))?;

        let client = Client {
            inner: Box::new(client),
        };

        let request = eth::SendBundleRequest {
            txs: vec![bytes!(
                "0x02f86b0180843b9aca00852ecc889a0082520894c87037874aed04e51c29f582394217a0a2b89d808080c080a0a463985c616dd8ee17d7ef9112af4e6e06a27b071525b42182fe7b0b5c8b4925a00af5ca177ffef2ff28449292505d41be578bebb77110dfc09361d2fb56998260"
            )],
            block_number: U64::from(0x1),
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
            replacement_uuid: None,
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
