# Trading Engine - Project Documentation

## Project Overview

A high-performance **order matching engine** built in Rust with a REST API powered by Actix-web. This system implements a centralized order book for financial trading pairs (crypto, forex, stocks, etc.), handling both market orders and limit orders with automatic matching.

**Tech Stack:**
- **Language:** Rust 2021 edition
- **Web Framework:** Actix-web 4.x
- **Decimal Precision:** rust_decimal 1.31.0
- **Serialization:** Serde + serde_json

---

## Architecture Overview

### Core Components

#### 1. **MatchEngine** (`src/order_matching_engine/engine.rs`)
- **Role:** Central orchestrator for all trading operations
- **Data Structure:** HashMap of trading pairs to order books
- **Key Responsibilities:**
  - Manages multiple trading pairs (e.g., BTC/USD, BTC/ETH)
  - Routes market and limit orders to appropriate order books
  - Validates trading pair existence

**Key Methods:**
- `new()` - Initializes empty engine
- `add_new_market(pair)` - Creates new trading pair market
- `fill_market_order(pair, order)` - Executes market order against best available asks/bids
- `place_limit_order(pair, price, order)` - Places limit order at specified price with validation
- `get_limits_for_a_pair(pair)` - Retrieves order book for visualization

#### 2. **OrderBook** (`src/order_matching_engine/orderbook.rs`)
- **Role:** Maintains bid/ask sides of a single trading pair
- **Current Data Structure:** **Binary Tree (BTreeMap)** for each side
  - `asks: BTreeMap<Decimal, Limit>` - Sorted sell orders by price
  - `bids: BTreeMap<Decimal, Limit>` - Sorted buy orders by price
- **Capacity Tracking:**
  - `ask_capacity` - Total volume available for purchase
  - `bid_capacity` - Total volume available for sale

**Key Methods:**
- `add_limit_order(price, order)` - Adds limit order at specific price level
- `fill_order_book(market_order)` - Matches market order against available limits
- `ask_limits()` / `bid_limits()` - Returns mutable references to sorted orders

#### 3. **Limit** (Price Level)
- **Role:** Container for all orders at a specific price point
- **Fields:**
  - `price: Decimal` - Price level
  - `orders: Vec<Order>` - Queue of orders at this price
  - `total_volume: f64` - Aggregated volume at this level

**Key Methods:**
- `add_order(order)` - Appends order to queue
- `fill_order(market_order)` - Matches orders FIFO (First In, First Out)

#### 4. **Order**
- **Role:** Individual trade request
- **Fields:**
  - `size: f64` - Volume to trade
  - `bid_or_ask: BidOrAsk` - Bid (buy) or Ask (sell) side

#### 5. **TradingPair**
- **Role:** Unique identifier for a market
- **Fields:**
  - `base: String` - Base currency (e.g., "btc")
  - `quote: String` - Quote currency (e.g., "usd")
- **Format:** `{base}_{quote}` (e.g., "btc_usd")

---

## REST API Endpoints

### 1. Create Market Order
```
POST /create_market_order/{base}_{quote}/{buy_or_sell}/{size}
```
Immediately matches against best available orders.

**Example:** `POST /create_market_order/btc_usd/buy/2.5`
- Creates a market buy order for 2.5 BTC at best available ask price

**Response:** Success/error message with fill details

---

### 2. Create Limit Order
```
POST /create_limit_order/{base}_{quote}/{buy_or_sell}/{price}/{size}
```
Places order at specific price level. Validated to prevent self-matching.

**Example:** `POST /create_limit_order/btc_usd/sell/50000.50/1.0`
- Creates a limit sell order for 1 BTC at $50,000.50

**Validation Logic:**
- **Buy Order:** Cannot place above best ask (prevents crossing spread)
- **Sell Order:** Cannot place below best bid (prevents crossing spread)
- Returns error if validation fails

**Response:** Confirmation message with order details

---

### 3. Get List of Trading Pairs
```
GET /get_list_of_pairs
```
Returns array of all available trading pairs.

**Example Response:**
```json
[["btc", "usd"], ["btc", "eth"]]
```

---

### 4. Get Order Book for Pair
```
GET /get_limits_for_a_pair/{base}_{quote}
```
Retrieves full order book (all bids and asks) for a pair.

**Example:** `GET /get_limits_for_a_pair/btc_usd`

**Response:** Complete OrderBook JSON with all price levels and volumes

---

## Order Matching Logic

### Market Order Matching Flow

1. **Order Validation:** Check trading pair exists
2. **Side Selection:** Route to opposite side
   - Buy market order → matches against ask limits (lowest first)
   - Sell market order → matches against bid limits (highest first)
3. **FIFO Matching:** At each price level, match orders in queue order
4. **Partial Fills:** Orders partially fill if insufficient volume
5. **Cleanup:** Remove exhausted price levels
6. **Capacity Update:** Reduce available volume on matched side

### Limit Order Placement Flow

1. **Price Validation:**
   - Bid must be below best ask (if asks exist)
   - Ask must be above best bid (if bids exist)
2. **Addition:** Insert order into price level queue
   - If price level exists: append to queue (FIFO)
   - If price level new: create level with order
3. **Capacity Update:** Increase opposite-side capacity
   - Placing bid increases ask_capacity
   - Placing ask increases bid_capacity

---

## Current Implementation: Binary Tree (BTreeMap)

### ✅ **Advantages:**

1. **Automatic Sorting:** BTreeMap maintains orders sorted by price
   - Asks sorted ascending (lowest ask first)
   - Bids sorted descending (highest bid first)
   
2. **Efficient Range Queries:** Can query price ranges efficiently
   - Good for advanced features (trade reports by price range)

3. **Built-in Ordering:** Natural semantics for financial data
   - Best bid/ask access is O(1) via `first()` / `last()`

4. **Memory Efficient:** For sparse price levels, better than array-based structures

### ❌ **Disadvantages:**

1. **Insertion Complexity:** O(log N) per order insertion
   - Add latency to order placement
   - Multiple orders at same price benefit less than alternatives

2. **Unnecessary Comparison Overhead:** 
   - BTreeMap performs red-black tree rotations
   - Overkill for adding to price levels

3. **Cache Efficiency:** BTreeMap doesn't have the locality of HashMap
   - More CPU cache misses for tight loops

4. **Not Optimized for FIFO:** FIFO matching (at price level) uses Vec anyway
   - BTreeMap overhead isn't leveraged for within-level matching

---

## Recommended Alternative: HashMap + Vec for Price Levels

### Architecture Comparison

**Current (BTreeMap):**
```
OrderBook {
  asks: BTreeMap<Decimal, Limit>  // Sorted by key
  bids: BTreeMap<Decimal, Limit>  // Sorted by key
}
```

**Recommended (HashMap + Vec):**
```
OrderBook {
  asks: HashMap<Decimal, Limit>      // Fast O(1) lookup/insert
  ask_prices: Vec<Decimal>            // Maintains sorted order
  bids: HashMap<Decimal, Limit>       // Fast O(1) lookup/insert
  bid_prices: Vec<Decimal>            // Maintains sorted order (descending)
}
```

### Rationale

1. **Insertion Speed:** O(1) HashMap insertion vs O(log N) BTreeMap
   - For order matching, insertion is critical path
   - Price levels are typically stable; prices don't change mid-session

2. **Sorted Access:** Vec maintains order explicitly
   - Only sort when needed (new price level added)
   - Sorting happens infrequently compared to order additions

3. **FIFO Matching:** Leverages existing Vec in Limit
   - HashMap → direct access to price level
   - Vec within Limit → FIFO order preservation

4. **Real-world Trading Patterns:**
   - Most activity at best bid/ask
   - Price levels change infrequently
   - Sorting cost amortized over many orders

### Implementation Pseudocode

```rust
pub struct OrderBook {
    asks: HashMap<Decimal, Limit>,
    ask_prices: Vec<Decimal>,  // sorted ascending
    
    bids: HashMap<Decimal, Limit>,
    bid_prices: Vec<Decimal>,  // sorted descending
    
    ask_capacity: f64,
    bid_capacity: f64,
}

impl OrderBook {
    pub fn add_limit_order(&mut self, price: Decimal, order: Order) {
        match order.bid_or_ask {
            BidOrAsk::Ask => {
                if !self.asks.contains_key(&price) {
                    self.ask_prices.push(price);
                    self.ask_prices.sort(); // O(N log N) but N = price levels << order count
                }
                self.asks.entry(price)
                    .or_insert_with(|| Limit::new(price))
                    .add_order(order);
                self.bid_capacity += order.size;
            }
            BidOrAsk::Bid => {
                if !self.bids.contains_key(&price) {
                    self.bid_prices.push(price);
                    self.bid_prices.sort_by(|a, b| b.cmp(a)); // descending
                }
                self.bids.entry(price)
                    .or_insert_with(|| Limit::new(price))
                    .add_order(order);
                self.ask_capacity += order.size;
            }
        }
    }
    
    pub fn first_price_ask(&self) -> Option<Decimal> {
        self.ask_prices.first().copied()
    }
    
    pub fn first_price_bid(&self) -> Option<Decimal> {
        self.bid_prices.first().copied()
    }
}
```

### Performance Comparison

| Operation | BTreeMap | HashMap + Vec |
|-----------|----------|--------------|
| Add order at new price | O(log P) + creation | O(1) + O(P log P) sort* |
| Add order at existing price | O(log P) tree walk | O(1) |
| Get best bid/ask | O(log P) | O(1) |
| Fill market order | O(log P) per level | O(1) per level |
| Remove empty level | O(log P) | O(1) + O(P) to rebuild vec |

*Price level sorting amortized: only N_new_prices sorts vs. continuous rotations

---

## Current Limitations & Future Improvements

### 1. **No Order Cancellation**
   - Orders cannot be modified or cancelled after placement
   - Risk: User stuck with unwanted orders

### 2. **No Partial Fill Awareness**
   - API doesn't return how much was filled vs. remaining
   - Risk: Client confusion on order status

### 3. **Basic Capacity Tracking**
   - `ask_capacity` / `bid_capacity` not always accurate during fills
   - Note: There's a logic error in `add_limit_order` - bids increase ask_capacity

### 4. **No Order History**
   - No audit trail of executed trades
   - Risk: Compliance/reconciliation issues

### 5. **Naive FIFO Matching**
   - All orders at same price treated equally
   - Real exchanges often use pro-rata or auction matching

### 6. **No Time Handling**
   - No order time-to-live (TTL) or good-til-cancelled (GTC) logic
   - All orders persist indefinitely

### 7. **Single-threaded Matching**
   - Mutex lock blocks all operations during matching
   - Could benefit from lock-free data structures for high frequency

---

## Testing

### Test Coverage

Located in `src/order_matching_engine/testing/tests.rs`

**Key Test Cases:**
1. **order_book_test()** - End-to-end bid/ask matching
2. **total_volume_test()** - Volume tracking accuracy
3. **total_volume_test2()** - Multiple orders per level
4. **total_volume_test3()** - Partial fill scenarios
5. **limit_order_fill()** - FIFO matching correctness

### Running Tests

```bash
cargo test
```

---

## Deployment

### Docker Build

A `Dockerfile` is provided for containerized deployment.

```bash
docker build -t trading-engine:latest .
docker run -p 8080:8080 trading-engine:latest
```

### Local Development

```bash
cargo run
```

Server listens on `0.0.0.0:8080`

---

## Code Quality Notes

### Strengths
✅ Type-safe order matching  
✅ Decimal precision (avoids float rounding errors)  
✅ Clean separation of concerns  
✅ Serializable data structures  

### Areas for Improvement
⚠️ Mutable references in `bid_limits()` / `ask_limits()` are error-prone  
⚠️ Logic error: capacity tracking has bid/ask reversed  
⚠️ No explicit error types (using strings)  
⚠️ Manual panic risk in `.unwrap()` calls  
⚠️ No concurrent order support (Mutex blocks)  

---

## Recommendation Summary

### Switch from BTreeMap to HashMap + Vec if:
- **Order insertion latency is critical** (sub-millisecond matching required)
- **Price levels are sparse** (few distinct prices in order book)
- **Throughput matters more than latency** (high order/sec volume)

### Keep BTreeMap if:
- **Simplicity preferred** (current code works correctly)
- **Price range queries needed** (future analytics features)
- **Order volume is low** (<100 orders/sec per pair)

### Hybrid Recommendation (Best of Both)
- Use **HashMap + Vec** for production matching engine
- Add **BTreeMap mirror** for reporting/analytics
- Implement order cancellation first
- Fix capacity tracking logic bugs

---

## Summary

This trading engine provides a **solid foundation** for matching orders with:
- ✅ Correct FIFO semantics
- ✅ Type-safe decimal arithmetic
- ✅ REST API integration
- ✅ Multi-pair support

**For optimization:** Consider HashMap + Vec for O(1) insertion at the cost of manual price level management.

**For reliability:** Fix capacity tracking bugs and add comprehensive error handling before production use.

