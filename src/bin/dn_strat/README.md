# Delta Neutral Strategy

A delta neutral funding rate farming strategy for Hyperliquid.

## Strategy Overview

**Long Spot + Short Perp** when funding rate is positive to capture funding payments while maintaining delta neutrality.

## Configuration

Set environment variables:

```bash
PRIVATE_KEY=your_private_key
USER_ADDRESS=0x1234...
BOT_URL=https://your-webhook-url.com
CHECK_EVERY=60  # seconds
BIND_ADDR=0.0.0.0:3000
```

## Running

```bash
cargo run --bin strat
```

## Strategy Logic

1. **Entry**: Long spot + Short perp when funding rate > 0
2. **Monitoring**: Continuous health checks every `CHECK_EVERY` seconds
3. **Exit Conditions**:
   - Funding rate turns negative
   - Price approaches liquidation threshold (70% of liquidation price)
4. **Notifications**: Webhook alerts for strategy events

## API Endpoints

The strategy exposes HTTP endpoints for manual control:

- `POST /v1/strategy/open` - Enter position
- `POST /v1/strategy/close` - Exit position
- `GET /v1/strategy/position` - Get current position status

## Risk Management

- **Liquidation Risk**: Monitors price vs liquidation price
- **Funding Risk**: Exits when funding turns negative
- **Dust Threshold**: Ignores positions below 0.1 USD
- **Slippage**: 0.5% slippage tolerance

## Strategy Parameters

- **Leverage**: 1x (configurable)
- **Slippage**: 0.5%
- **Dust Threshold**: 0.1 USD
- **Liquidation Threshold**: 70% of liquidation price
