use alloy::{
    providers::{ProviderBuilder, WsConnect},
    signers::local::PrivateKeySigner,
};
use anyhow::Result;
use clap::Parser;
use kazuka_core::{
    engine::Engine, event_sources::mev_share_event_source::MevShareEventSource,
    types::EventSourceMap,
};
use kazuka_mev_share_arbitrage::{
    strategy::MevShareUniswapV2V3Arbitrage,
    types::{Action, Event},
};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// CLI options.
#[derive(Parser, Debug)]
struct Args {
    /// Ethereum node WS endpoint.
    #[arg(long)]
    pub wss: String,
    /// Private key for sending txs.
    #[arg(long)]
    pub tx_signer_pk: String,
    /// Private key for MEV-Share signer.
    #[arg(long)]
    pub flashbots_signer_pk: String,
    /// Address of the arbitrage contract.
    #[arg(long)]
    pub arb_contract_address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let target_filter = tracing_subscriber::filter::Targets::new()
        .with_target("kazuka_core", Level::TRACE)
        .with_target("kazuka_mev_share_backend", Level::TRACE)
        .with_target("kazuka_mev_share_sse", Level::TRACE)
        .with_target(
            "kazuka_mev_share_arbitrage",
            Level::INFO,
        );

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(target_filter)
        .init();

    let args = Args::parse();

    let ws = WsConnect::new(args.wss);

    let tx_signer: PrivateKeySigner = args.tx_signer_pk.parse()?;
    let provider = ProviderBuilder::new()
        .wallet(tx_signer)
        .connect_ws(ws)
        .await?;

    let flashbots_signer: PrivateKeySigner =
        args.flashbots_signer_pk.parse()?;

    let engine: Engine<Event, Action> = Engine::default();

    let mev_share_event_source =
        MevShareEventSource::new("https://mev-share.flashbots.net".to_string());
    let mev_share_event_source = EventSourceMap::new(
        Box::new(mev_share_event_source),
        Event::MevShareEvent,
    );
    engine.add_event_source(Box::new(mev_share_event_source));

    // let strategy = MevShareUniswapV2V3Arbitrage::new()

    Ok(())
}
