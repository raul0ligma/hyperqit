use std::time::Duration;
use std::{env, sync::Arc};

use alloy::primitives::Address;
use envconfig::Envconfig;
use hyperqit::*;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::info;

mod config;
mod handlers;
mod notifier;
mod router;
mod service;
mod strategy;

use config::Config;
use notifier::NotifierService;
use router::create_router;
use service::StrategyManagerService;
use strategy::{Asset, Strategy};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let config = Config::init_from_env().unwrap();
    let signer = Signers::Local(hyperqit::LocalWallet::signer(config.private_key));

    let user_address: Address = config.user_address.parse().unwrap();

    let executor = crate::HyperliquidClient::new(Network::Testnet, signer, user_address);
    let asset = Asset::CommonAsset("HYPE".to_owned());
    let notifier = NotifierService::new(config.bot_url, user_address.to_string());
    let strategy = Arc::new(Strategy::new(
        1,
        Duration::from_secs(config.check_every),
        asset.clone(),
        0.005,
        0.1f64,
        0.7,
        executor,
        notifier,
    ));

    let strategy_manager = Arc::new(StrategyManagerService::new(
        strategy.clone(),
        asset,
        user_address.to_string(),
    ));
    let app = create_router(strategy_manager);

    let listener = tokio::net::TcpListener::bind(config.bind_addr.clone())
        .await
        .unwrap();
    info!("server running on http://{}", config.bind_addr);

    let cancellation = CancellationToken::new();
    let strategy_for_runner = strategy.clone();
    let server_cancellation = cancellation.clone();
    let runner_cancellation = cancellation.clone();

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                server_cancellation.cancelled().await;
                println!("server shutting down gracefully...");
            })
            .await
            .unwrap();
    });

    let runner_handle =
        tokio::spawn(async move { strategy_for_runner.run(runner_cancellation).await.unwrap() });

    let signal_cancellation = cancellation.clone();
    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt()).unwrap();
        let mut sigquit = signal::unix::signal(signal::unix::SignalKind::quit()).unwrap();

        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("received SIGINT");
            }
            _ = sigterm.recv() => {
                info!("received SIGTERM");
            }
            _ = sigint.recv() => {
                info!("received SIGINT");
            }
            _ = sigquit.recv() => {
                info!("Received SIGQUIT");
            }
        }

        println!("initiating graceful shutdown");
        signal_cancellation.cancel();
    });

    let _ = tokio::join!(server_handle, runner_handle);
    info!("graceful shutdown complete");
}
