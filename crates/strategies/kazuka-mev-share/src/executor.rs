use alloy::signers::Signer;
use jsonrpsee::http_client::{
    HttpClient, HttpClientBuilder,
    transport::{self},
};
use mev_share::rpc::{FlashbotsSignerLayer, MevApiClient};

pub struct MevShareExecutor {
    client: Box<dyn MevApiClient + Send + Sync>,
}

impl MevShareExecutor {
    pub fn new(signer: impl Signer + Clone + 'static) -> Self {
        // let middleware = FlashbotsSignerLayer::new(signer);

        // let http = HttpClient::builder()
        //     .set_http_middleware(middleware)
        //     .build("https://relay.flashbots.net:443");
        // .expect("failed to build HTTP client");
        //

        todo!()
    }
}

