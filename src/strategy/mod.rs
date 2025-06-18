use std::time::Duration;

use log::info;
use tokio::time;
use tokio_util::sync::CancellationToken;

use crate::{HlAgentWallet, HyperliquidClient, errors::Result};
pub struct Strategy<S>
where
    S: HlAgentWallet,
{
    leverage: u32,
    tick_interval: Duration,
    executor: HyperliquidClient<S>,
}

impl<S> Strategy<S>
where
    S: HlAgentWallet,
{
    pub fn new(leverage: u32, tick_interval: Duration, executor: HyperliquidClient<S>) -> Self {
        Strategy {
            leverage: leverage,
            tick_interval: tick_interval,
            executor: executor,
        }
    }

    pub async fn run(&self, cancellation: CancellationToken) -> Result<()> {
        info!("starting strategy runner");
        let mut ticker = time::interval(self.tick_interval);

        loop {
            tokio::select! {
                tick_out = ticker.tick() =>{
                        info!("running strategy {:?}", tick_out.elapsed());
                }
                _ = cancellation.cancelled() =>{
                        println!("cancelled");
                        break;
                }
            }
        }

        Ok(())
    }
}
