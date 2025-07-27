use std::sync::Arc;

use alloy::{network::AnyNetwork, providers::DynProvider, signers::Signer};
use jsonrpsee::http_client::{
    HttpClient, HttpClientBuilder,
    transport::{self},
};
use mev_share::rpc::{FlashbotsSignerLayer, MevApiClient};

pub struct MevShareExecutor {
    provider: Arc<DynProvider<AnyNetwork>>,
}

impl MevShareExecutor {
    pub fn new(provider: Arc<DynProvider<AnyNetwork>>) -> Self {
        Self { provider }
    }
}

impl Executor<SendBundleRequest> for MevShareExecutor {}
