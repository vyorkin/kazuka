use async_trait::async_trait;
use jsonrpsee::proc_macros::rpc;

use crate::{
    EthBundleHash, EthSendBundle, SendBundleRequest, SendBundleResponse,
};

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
        /// `eth_sendBundle` can be used to send your bundles to the builder.
        #[method(name = "sendBundle")]
        async fn send_bundle(
            &self,
            request: EthSendBundle,
        ) -> RpcResult<EthBundleHash>;
    }
}
