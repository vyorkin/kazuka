use std::sync::Arc;

use alloy::{
    primitives::Address,
    providers::{ProviderBuilder, WsConnect},
    signers::local::PrivateKeySigner,
};
use anyhow::Result;
use clap::Parser;
use kazuka_core::{
    engine::Engine,
    event_sources::mev_share_event_source::MevShareEventSource,
    types::{EventSourceMap, ExecutorMap},
};
use kazuka_mev_share_arbitrage::{
    executor::MevShareExecutor,
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
    /// Whether to actually submit bundles or just log them.
    #[arg(long, action)]
    pub dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let target_filter = tracing_subscriber::filter::Targets::new()
        .with_target("kazuka_simple_arbitrage", Level::INFO)
        .with_target("kazuka_core", Level::INFO)
        .with_target("kazuka_mev_share_backend", Level::INFO)
        .with_target("kazuka_mev_share_sse", Level::INFO)
        .with_target(
            "kazuka_mev_share_arbitrage",
            Level::INFO,
        );

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_ansi(true).pretty())
        .with(target_filter)
        .init();

    let args = Args::parse();

    let ws = WsConnect::new(args.wss);

    tracing::info!("Strating probablistic blind arbitrage strategy...");

    let tx_signer: PrivateKeySigner = args.tx_signer_pk.parse()?;
    let provider = ProviderBuilder::new()
        .wallet(tx_signer.clone())
        .connect_ws(ws)
        .await?;

    let flashbots_signer: PrivateKeySigner =
        args.flashbots_signer_pk.parse()?;

    let provider = Arc::new(provider);

    let mev_share_event_source =
        MevShareEventSource::new("https://mev-share.flashbots.net".to_string());
    let mev_share_event_source = EventSourceMap::new(
        Box::new(mev_share_event_source),
        Event::MevShareEvent,
    );

    let arbitrage_contract_address =
        Address::parse_checksummed(args.arb_contract_address, None)?;
    let strategy = MevShareUniswapV2V3Arbitrage::new(
        provider,
        arbitrage_contract_address,
        args.dry_run,
    );

    let mev_share_executor = MevShareExecutor::new(
        "https://relay.flashbots.net:443".to_string(),
        args.dry_run,
        flashbots_signer,
    );
    let mev_share_executor = ExecutorMap::new(
        Box::new(mev_share_executor),
        |action| match action {
            Action::SubmitBundle(bundle) => Some(bundle),
        },
    );

    let engine: Engine<Event, Action> = Engine::default()
        .add_event_source(Box::new(mev_share_event_source))
        .add_strategy(Box::new(strategy))
        .add_executor(Box::new(mev_share_executor));

    let result = match engine.run().await {
        Ok(mut set) => {
            while let Some(result) = set.join_next().await {
                tracing::info!("result: {:?}", result);
            }
            Ok(())
        }
        Err(err) => {
            tracing::error!("Error running engine: {:?}", err);
            Err(err.into())
        }
    };

    tracing::info!("All done! Exiting...");

    result
}
