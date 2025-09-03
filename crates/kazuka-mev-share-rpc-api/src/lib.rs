//! MEV-Share RPC interface definitions.

mod eth;
mod flashbots;
mod mev;
pub mod middleware;
pub mod types;

#[cfg(feature = "client")]
pub use clients::*;
#[cfg(feature = "server")]
pub use servers::*;

#[cfg(feature = "server")]
pub mod servers {
    pub use crate::{
        eth::EthBundleApiServer, flashbots::FlashbotsApiServer,
        mev::MevApiServer,
    };
}
#[cfg(feature = "client")]
pub mod clients {
    pub use crate::{
        eth::EthBundleApiClient, flashbots::FlashbotsApiClient,
        mev::MevApiClient,
    };
}
