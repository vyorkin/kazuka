use alloy::signers::Signer;
use jsonrpsee::http_client::{
    HttpClient, HttpClientBuilder,
    transport::{self},
};
use mev_share::rpc::{FlashbotsSigner, FlashbotsSignerLayer, MevApiClient};

pub struct MevShareExecutor {
    client: Box<dyn MevApiClient + Send + Sync>,
}

impl MevShareExecutor {
    pub fn new(signer: impl Signer + Clone + 'static) -> Self {
        // let provider

        todo!()
    }
}

// impl MevShareExecutor {
//     pub fn new(signer: impl Signer + Clone + 'static) -> Self {
//         let signer_layer = FlashbotsSignerLayer::new(signer);
//         let service_builder = tower::ServiceBuilder::new()
//             .map_err(transport::Error::Http)
//             .layer(signer_layer);
//
//         let http =
//
//         // HttpClient::builder().set_rpc_middleware(service_builder);
//
//         // .build("https://relay.flashbots.net:443")
//         // .unwrap();
//
//         // .set_http_middleware(service_builder);
//         // .build("https://relay.flashbots.net:443")
//         // .expect("failed to build HTTP client");
//
//         todo!()
//     }
// }
