# hlmm

A Rust-based automated trading bot for Hyperliquid, designed for robust, configurable strategy execution and risk management.

## Features

- **Strategy Engine:**

  - Periodically evaluates market and funding conditions.
  - Dynamically manages positions in spot and perpetual markets.
  - Supports configurable leverage, slippage, and liquidation risk thresholds.

- **Health and Risk Monitoring:**

  - Monitors funding rates and proximity to liquidation.
  - Automatically exits or maintains positions based on real-time risk assessment.

- **Position Management:**

  - Programmatic entry and exit of positions.
  - Unified handling of spot and perp assets.
  - Automated rebalancing and state tracking.

- **Hyperliquid Integration:**
  - Direct interaction with Hyperliquid APIs for order placement, funding history, leverage updates, and asset transfers.
  - Secure signing and wallet management.

## Strategy Logic

- **Risk-Aware Execution:**
  - Enters positions according to user configuration.
  - Exits positions if funding turns negative or liquidation risk exceeds threshold.
  - Maintains positions when conditions are favorable, with continuous monitoring.

## Configuration

- All operational parameters (keys, asset, thresholds) are set via environment variables.
