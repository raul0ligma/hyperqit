# **Hyperliquid Multi-Strategy Bot**

## **The Idea**

Switch between delta neutral and market making based on funding conditions. When funding is positive, collect funding payments. When funding is negative, run market making with buy bias.

## **Strategy Logic**

- **Positive funding** → Long spot + short perp (collect funding payments)
- **Negative funding** → Market making with buy bias (accumulate during pessimism)
- **Check hourly** for funding payments, switch every 8 hours on rate updates

## **Core Components**

### **Strategy Switching Engine**

- Monitor funding rates
- Transition between strategies
- Handle position changes

### **Delta Neutral Strategy**

- Long spot position
- Equal short perp position
- Automatic rebalancing
- Funding collection

### **Market Making Strategy**

- Place bid/ask orders around mid
- Buy bias during negative funding
- Inventory management
- Spread capture

### **Risk Management**

- Position size limits
- Daily loss limits
- Emergency exit logic

## **Implementation Order**

1. Basic market maker with configuration
2. Delta neutral strategy with funding monitoring
3. Strategy switching logic
4. Risk management and observability
5. Testing and optimization

## **Key Decisions**

- Funding rate thresholds for switching
- Market making bias levels
- Position sizing and risk limits
- Rebalancing frequency
