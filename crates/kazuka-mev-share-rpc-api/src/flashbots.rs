use alloy::{
    primitives::{B256, U64},
    rpc::types::mev::{BundleStats, UserStats},
};
use async_trait::async_trait;
use jsonrpsee::{core::ClientError, proc_macros::rpc};
use tracing::instrument;

use crate::types::{GetBundleStatsRequest, GetUserStatsRequest};

/// Generates a client using jsonrpsee proc macros.
///
/// This hides the generated client trait which is
/// replaced by the `FlashbotsApiClient` trait.
///
/// [jsonrpsee_proc_macros]: https://docs.rs/jsonrpsee-proc-macros/latest/jsonrpsee_proc_macros/attr.rpc.html
mod rpc {
    use jsonrpsee::core::RpcResult;

    use super::*;

    /// Flashbots RPC interface.
    #[cfg_attr(not(feature = "server"), rpc(client, namespace = "mev"))]
    #[cfg_attr(not(feature = "client"), rpc(server, namespace = "mev"))]
    #[cfg_attr(
        all(feature = "client", feature = "server"),
        rpc(client, server, namespace = "mev")
    )]
    #[async_trait]
    pub trait FlashbotsApi {
        /// See [`super::FlashbotsApiClient::get_user_stats`]
        #[method(name = "getUserStatsV2")]
        async fn get_user_stats(
            &self,
            request: GetUserStatsRequest,
        ) -> RpcResult<UserStats>;

        /// See [`super::FlashbotsApiClient::get_user_stats`]
        #[method(name = "getBundleStatsV2")]
        async fn get_bundle_stats(
            &self,
            request: GetBundleStatsRequest,
        ) -> RpcResult<BundleStats>;
    }
}

// Re-export the rpc server trait.
#[cfg(feature = "server")]
pub use rpc::FlashbotsApiServer;

/// An dyn-trait compatible (vtable compatible) version of the
/// `FlashbotsApiClient` trait.
#[cfg(feature = "client")]
#[async_trait]
pub trait FlashbotsApiClient {
    /// Returns a quick summary of how a searcher is performing in the Flashbots
    /// ecosystem, including their reputation-based priority.
    ///
    /// Note: It is currently updated once every hour.
    ///
    /// # Arguments
    ///
    /// * `block_number` - A recent block number, in order to prevent replay
    ///   attacks. Must be within 20 blocks of the current chain tip.
    async fn get_user_stats(
        &self,
        block_number: U64,
    ) -> Result<UserStats, ClientError>;

    /// Returns stats for a single bundle.
    ///
    /// You must provide a blockNumber and the bundleHash, and the signing
    /// address must be the same as the one who submitted the bundle.
    ///
    /// # Arguments
    ///
    /// * `block_hash` - Returned by the Flashbots API when calling
    ///   `eth_sendBundle`/`mev_sendBundle`.  [`crate::SendBundleResponse`].
    /// * `block_number` - The block number the bundle was targeting. See
    ///   [`crate::Inclusion`].
    async fn get_bundle_stats(
        &self,
        bundle_hash: B256,
        block_number: U64,
    ) -> Result<BundleStats, ClientError>;
}

#[cfg(feature = "client")]
#[async_trait::async_trait]
impl<T> FlashbotsApiClient for T
where
    T: rpc::FlashbotsApiClient + Sync,
{
    /// See [`FlashbotsApiClient::get_user_stats`]
    #[instrument(skip(self))]
    async fn get_user_stats(
        &self,
        block_number: U64,
    ) -> Result<UserStats, ClientError> {
        self.get_user_stats(GetUserStatsRequest { block_number })
            .await
    }

    /// See [`FlashbotsApiClient::get_user_stats`]
    #[instrument(skip(self))]
    async fn get_bundle_stats(
        &self,
        bundle_hash: B256,
        block_number: U64,
    ) -> Result<BundleStats, ClientError> {
        self.get_bundle_stats(GetBundleStatsRequest {
            bundle_hash,
            block_number,
        })
        .await
    }
}
