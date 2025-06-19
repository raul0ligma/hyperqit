use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use alloy::primitives::Address;
use envconfig::Envconfig;
use hlmm::*;
use log::info;
use tokio::signal;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = hlmm::Config::init_from_env().unwrap();
    let signer = Signers::Local(hlmm::LocalWallet::signer(config.private_key.into()));

    let user_address: Address = config.user_address.parse().unwrap();

    let executor = crate::HyperliquidClient::new(Network::Mainnet, signer, user_address);

    let asset = Asset::CommonAsset("HYPE".to_owned());
    let strategy = Arc::new(Strategy::new(
        1,
        Duration::from_secs(10),
        asset.clone(),
        0.005,
        0.1f64,
        0.7,
        executor,
    ));

    let strategy_manager = Arc::new(StrategyManagerService::new(
        strategy.clone(),
        asset,
        user_address.to_string(),
    ));
    let app = create_router(strategy_manager);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");

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
