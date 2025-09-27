# hyperqit

A Rust SDK for Hyperliquid.

## Installation

```bash
cargo add hyperqit
```

## Quick Start

### Placing Orders

```rust
use hyperqit::*;
use alloy::primitives::Address;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup signer
    let private_key = "your_private_key_here";
    let signer = LocalWallet::signer(private_key.to_string());
    let user_address = signer.address();

    // Create client
    let client = HyperliquidClient::new(
        Network::Testnet,
        Box::new(signer),
        user_address,
    );

    // Place an order
    let order = BulkOrder {
        orders: vec![OrderRequest {
            asset: 0, // BTC perpetual
            is_buy: true,
            limit_px: "50000.0".to_string(),
            sz: "0.001".to_string(),
            reduce_only: false,
            order_type: OrderType::Limit(Limit { tif: "Ioc".into() }),
            cloid: None,
        }],
        grouping: "na".to_string(),
    };

    let result = client.create_position_raw(order).await?;
    println!("Order placed: {:?}", result);

    Ok(())
}
```

### Multi Sig Operations

```rust
use hyperqit::*;
use alloy::primitives::Address;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup primary signer
    let primary_signer = LocalWallet::signer("primary_key".to_string());
    let primary_address = primary_signer.address();

    // Setup other signers
    let other_signers = vec![
        Box::new(LocalWallet::signer("signer2_key".to_string())),
        Box::new(LocalWallet::signer("signer3_key".to_string())),
    ];

    let client = HyperliquidClient::new(
        Network::Testnet,
        Box::new(primary_signer),
        primary_address,
    );

    // Multi-sig USD transfer
    let multi_sig_address: Address = "0x1234...".parse()?;
    client.multi_sig_usd_class_transfer(
        1000,                    // amount
        true,                    // to_perp
        "0xa4b1".to_string(),    // chain ID
        other_signers,           // other signers
        multi_sig_address,       // multi-sig address
    ).await?;

    println!("Multi-sig transfer completed");
    Ok(())
}
```

## Asset IDs & Market Data

Get asset IDs and market information:

```rust
use hyperqit::market_info::*;

// Get unified market data
let perp_info = client.get_perp_info(None).await?;
let spot_info = client.get_spot_info(None).await?;
let unified = create_unified_market_info(perp_info, spot_info);

// Find asset by name and get ID
if let Some(market) = find_market_by_name(&unified, "BTC") {
    let btc_perp_id = market.perp.as_ref().map(|p| p.asset_id);
    let btc_spot_id = market.spot.as_ref().map(|s| s.asset_id);

    println!("BTC Perp ID: {:?}", btc_perp_id);
    println!("BTC Spot ID: {:?}", btc_spot_id);
}

// Get current prices
let btc_perp_price = get_current_price(&unified, "BTC", true, true);  // perp, use_mid
let btc_spot_price = get_current_price(&unified, "BTC", false, true);  // spot, use_mid
```

## Examples

The binary tools demonstrate SDK usage:

- `dex` - DEX operations, order management, and market data
- `multisig` - Multi-signature operations and conversions
- `transfer` - USD transfers between spot and perp accounts
- `strat` - **Delta neutral funding rate farming strategy** with web interface
- `deployer` - **HIP-3 builder-deployed perpetuals** deployment and management

Run examples with:

```bash
cargo run --bin dex
cargo run --bin multisig
cargo run --bin transfer
cargo run --bin strat
cargo run --bin deployer
```

### Delta Neutral Strategy (`strat`)

Delta neutral funding rate farming strategy that goes long spot + short perp when funding is positive.

**Configuration:**

```bash
PRIVATE_KEY=your_private_key
USER_ADDRESS=0x1234...
BOT_URL=https://your-webhook-url.com
CHECK_EVERY=60
BIND_ADDR=0.0.0.0:3000
```

### HIP-3 Builder-Deployed Perpetuals (`deployer`)

The `deployer` binary demonstrates HIP-3 perpetual deployment using the SDK:

**SDK Support:**

- `PerpDeployAction` enum for all deployment actions
- `RegisterAsset` - Deploy new perpetual markets
- `SetOracle` - Update oracle and mark prices
- `SetFundingMultipliers` - Configure funding rates
- `HaltTrading` - Control market trading

**Example Usage:**

```rust
// Deploy new perpetual market
let deploy_action = PerpDeployAction::RegisterAsset(RegisterAsset {
    max_gas: Some(1000000),
    asset_request: RegisterAssetRequest {
        coin: "dex:NEWCOIN".to_string(),
        sz_decimals: 6,
        oracle_px: "100.0".to_string(),
        margin_table_id: 0,
        only_isolated: false,
    },
    dex: "dex".to_string(),
    schema: Some(PerpDexSchemaInput {
        full_name: "New Coin Perpetual".to_string(),
        collateral_token: 0,
        oracle_updater: None,
    }),
});

client.perp_deploy_action(deploy_action).await?;
```

## Signing

The SDK supports multiple signing methods through the `Signer` trait:

```rust
#[async_trait]
pub trait Signer {
    async fn sign_order(&self, to_sign: FixedBytes<32>) -> Result<SignedMessage>;
}
```

**Built-in Signers:**

- `LocalWallet` - Local private key signing

**Extending Signers:**
Implement the `Signer` trait for custom signing backends like hardware wallets, remote signers, or multi-party computation systems.

## License

MIT License
