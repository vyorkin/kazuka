use std::sync::Arc;

use alloy::{network::AnyNetwork, providers::DynProvider};

pub struct MempoolExecutor {
    provider: Arc<DynProvider<AnyNetwork>>,
}
