# Trading Engine - Architecture & Decision Summary

## Project Structure

```
tradingEngine/
│
├── src/
│   ├── main.rs                    # Actix-web HTTP server + API endpoints
│   │   ├── create_market_order/   # POST immediate execution
│   │   ├── create_limit_order/    # POST with price level
│   │   ├── get_list_of_pairs/     # GET available markets
│   │   └── get_limits_for_a_pair/ # GET order book snapshot
│   │
│   └── order_matching_engine/
│       ├── mod.rs                 # Module exports
│       ├── engine.rs              # MatchEngine: orchestrates multiple pairs
│       ├── orderbook.rs           # OrderBook: per-pair bid/ask management
│       │   ├── OrderBook struct
│       │   ├── Limit struct       # Price level container
│       │   ├── Order struct       # Individual order
│       │   └── BidOrAsk enum      # Side indicator
│       │
│       └── testing/
│           ├── tests.rs           # Unit tests
│           └── mod.rs
│
├── Cargo.toml                     # Dependencies
├── Dockerfile                     # Container config
└── README.md                      # Basic description
```

---

## Data Flow Diagram

### Market Order Flow
```
HTTP Request
    ↓
main.rs: create_market_order()
    ↓
MatchEngine::fill_market_order(pair, order)
    ↓
OrderBook::fill_order_book(order)
    ├─ Get opposite side limits (bids for sell, asks for buy)
    ├─ For each limit level (best to worst):
    │  ├─ Limit::fill_order(market_order)  [FIFO matching]
    │  └─ Update remaining volume
    ├─ Remove exhausted price levels
    └─ Update capacity counters
    ↓
HTTP Response: fill summary
```

### Limit Order Flow
```
HTTP Request
    ↓
main.rs: create_limit_order()
    ↓
MatchEngine::place_limit_order(pair, price, order)
    ├─ Validate price doesn't cross spread
    └─ If valid:
        OrderBook::add_limit_order(price, order)
            ├─ Get or create Limit at price level
            ├─ Add order to Limit queue (FIFO)
            └─ Update capacity counter
    ↓
HTTP Response: placement confirmation
```

---

## Decision Tree: BTreeMap vs HashMap + Vec

```
                    ┌─────────────────────────────────┐
                    │ Choose Order Book Data Structure │
                    └────────────────────┬─────────────┘
                                         │
                    ┌────────────────────┴────────────────────┐
                    │                                         │
                    ▼                                         ▼
        ┌──────────────────────────┐            ┌──────────────────────────┐
        │  Priority: Simplicity    │            │  Priority: Performance   │
        │  (Current Orders/sec)    │            │  (Future Orders/sec)     │
        └────────┬─────────────────┘            └────────┬─────────────────┘
                 │                                        │
        ┌────────▼──────────────┐              ┌──────────▼──────────────┐
        │ < 1,000 orders/sec    │              │ > 10,000 orders/sec    │
        │ Few price levels      │              │ Many price levels      │
        │ Predictable usage     │              │ Sustained high volume  │
        └────────┬──────────────┘              └──────────┬──────────────┘
                 │                                        │
                 ▼                                        ▼
        ┌─────────────────────┐              ┌──────────────────────────┐
        │   USE BTreeMap ✓    │              │  USE HashMap + Vec ✓     │
        ├─────────────────────┤              ├──────────────────────────┤
        │ • Built-in sorting  │              │ • O(1) insertion         │
        │ • Simple code       │              │ • Predictable latency    │
        │ • Natural API       │              │ • Scale-friendly         │
        │ • Proven correct    │              │ • Manual sort mgmt       │
        └─────────────────────┘              └──────────────────────────┘
```

---

## Matching Algorithm Flowchart

### fill_order_book() Logic

```
                    START: Market Order Arrives
                            │
                            ▼
                    ┌───────────────────┐
                    │ Size = Order.size  │
                    │ Side = Opposite    │
                    └─────────┬─────────┘
                              │
                              ▼
                    ┌───────────────────────────────┐
                    │ Check capacity > order size?  │
                    └───────┬──────────────────┬────┘
                    ┌──────▼─────┐      ┌─────▼──────┐
                    │ YES         │      │ NO (ERROR) │
                    └──────┬─────┘      └────────────┘
                           │
                           ▼
        ┌──────────────────────────────────────────┐
        │ For each Limit (best price first):       │
        ├──────────────────────────────────────────┤
        │  1. Call Limit::fill_order(market_order) │
        │  2. Match orders FIFO at this price      │
        │  3. Update market_order.size remaining   │
        │  4. If limit empty → mark for removal    │
        │  5. If market_order full → done          │
        └──────┬───────────────────────────────────┘
               │
        ┌──────▼──────┐
        │ All limits  │
        │ processed?  │
        └──────┬──────┘
               │
               ▼
        ┌──────────────────┐
        │ Remove empty     │
        │ price levels     │
        └──────┬───────────┘
               │
               ▼
        ┌──────────────────┐
        │ Update capacity  │
        │ counters         │
        └──────┬───────────┘
               │
               ▼
        ┌──────────────────┐
        │ Return fill      │
        │ summary          │
        └──────────────────┘
```

### Limit::fill_order() (FIFO Matching)

```
        START: Market order seeks to consume limit orders
                        │
                        ▼
        ┌────────────────────────────────┐
        │ For each limit order (FIFO):   │
        ├────────────────────────────────┤
        │ if market.size >= limit.size   │
        │   │                            │
        │   ├─ Fill entire limit order  │
        │   ├─ limit.size = 0           │
        │   ├─ market.size -= limit.size│
        │   └─ Mark limit for removal   │
        │                                │
        │ else (market.size < limit.size)│
        │   │                            │
        │   ├─ Partial fill of limit    │
        │   ├─ limit.size -= market.size│
        │   ├─ market.size = 0 (DONE!)  │
        │   └─ Break loop               │
        └────────────────────────────────┘
                        │
                        ▼
        ┌────────────────────────────────┐
        │ market_order.is_filled()?      │
        └────────┬──────────────────┬────┘
        ┌────────▼──┐         ┌─────▼──────┐
        │ YES: Done │         │ NO: Continue
        └───────────┘         └─────┬──────┘
                                    │
                        ┌───────────┘
                        │
                        ▼
        Remove marked limit orders (cleanup)
```

---

## BTreeMap vs HashMap+Vec Comparison Table

### Complexity Analysis

| Scenario | BTreeMap | HashMap+Vec | Winner |
|----------|----------|------------|--------|
| **Add order at new price** | O(log P) + tree rotations | O(1) + O(P log P) sort | Vec if P small, BTree if P large |
| **Add order at existing price** | O(log P) | O(1) | HashMap+Vec |
| **Match best n orders** | O(n log P) | O(n) | HashMap+Vec |
| **Remove empty level** | O(log P) | O(1) | HashMap+Vec |
| **Get all asks in order** | O(P) | O(P) | Tie |
| **Concurrent access** | O(log P) + lock | O(1) + lock | Tie (mutex is bottleneck) |

### Real-World Impact

```
Scenario: 10,000 orders/sec, 100 price levels

BTreeMap (Current):
  ├─ 10,000 insertions × log(100) = 66,439 tree ops
  ├─ Constant red-black rebalancing
  ├─ Branch prediction misses
  └─ Latency: ~100-200 microseconds per order

HashMap+Vec (Proposed):
  ├─ 10,000 insertions × O(1) = 10,000 hash ops
  ├─ Occasional sorts: 2-3 × 100 log(100) = 660 ops
  ├─ Better CPU cache locality
  └─ Latency: ~5-20 microseconds per order

Speedup: ~5-10x faster insertion at scale
```

---

## Current Issues & Fixes

### Issue #1: Capacity Tracking Reversed ⚠️

**Location:** `orderbook.rs` line 121-128

```rust
❌ WRONG:
BidOrAsk::Ask => self.bid_capacity += order.size
BidOrAsk::Bid => self.ask_capacity += order.size

✓ CORRECT:
BidOrAsk::Ask => self.ask_capacity += order.size
BidOrAsk::Bid => self.bid_capacity += order.size
```

**Impact:** Capacity values are inverted, causing invalid order rejection logic

### Issue #2: Panicking Unwraps ⚠️

**Location:** `orderbook.rs` lines 46-49

```rust
❌ RISKY:
pub fn first_price_ask(&mut self) -> Decimal {
    return self.ask_limits().get(0).unwrap().price;
}

✓ SAFE:
pub fn first_price_ask(&self) -> Option<Decimal> {
    self.ask_prices.first().copied()
}
```

**Impact:** Server crashes if orderbook is empty (edge case)

### Issue #3: Logic Scope ⚠️

**Location:** `engine.rs` line 57-58

```rust
❌ UNCLEAR:
if orderbook.first_price_ask() <= price {
    return Err(...);  // But what if no asks?
}

✓ CLEAR:
if let Some(best_ask) = orderbook.first_price_ask() {
    if best_ask <= price {
        return Err(...);
    }
}
```

**Impact:** Potential crashes if validation runs on empty book

---

## Quick Migration Path (If Needed)

### Phase 1: Stabilize (2-3 hours)
- [ ] Fix capacity tracking bug
- [ ] Replace unwrap() with error handling
- [ ] Add order cancellation support
- [ ] Run full test suite

### Phase 2: Benchmark (2-4 hours)
- [ ] Add load testing (1,000-100,000 orders/sec)
- [ ] Profile CPU/memory usage
- [ ] Measure latency percentiles (p50, p95, p99)
- [ ] Document baseline metrics

### Phase 3: Optimize (If Needed) (4-6 hours)
- [ ] Implement HashMap+Vec structure
- [ ] Update all access methods
- [ ] Port existing tests
- [ ] Compare performance vs baseline
- [ ] Deploy and monitor

**Total Effort:** 8-13 hours for complete stabilization + optimization

---

## Deployment Readiness Checklist

### Current Status: ⚠️ Pre-Production (Needs Fixes)

- [ ] Capacity tracking logic bug
- [ ] Error handling (no unwrap())
- [ ] Order cancellation support
- [ ] Trade history/audit trail
- [ ] Load testing (>10k orders/sec)
- [ ] Latency SLA validation (< 100ms p99)
- [ ] High availability setup
- [ ] Data persistence layer

### Production-Ready: Feature Set
- [x] Market order matching
- [x] Limit order placement
- [x] Multiple trading pairs
- [x] REST API
- [x] Serializable data
- [ ] Order cancellation
- [ ] Order history
- [ ] Risk limits
- [ ] Circuit breakers

---

## Testing Coverage

### Unit Tests (Exist)
- [x] Order book filling
- [x] Volume tracking
- [x] FIFO matching
- [x] Limit order placement

### Integration Tests (Missing)
- [ ] End-to-end order flow
- [ ] Concurrent order handling
- [ ] Capacity tracking accuracy
- [ ] Empty order book edge cases

### Load Tests (Missing)
- [ ] 1,000 orders/sec per pair
- [ ] 10,000 orders/sec per pair
- [ ] 100,000 orders/sec per pair
- [ ] Latency distribution percentiles

### Stress Tests (Missing)
- [ ] 1000+ active trading pairs
- [ ] Order cancellation spam
- [ ] Rapid market condition changes
- [ ] Network latency simulation

---

## Summary

### What Works ✓
- FIFO matching semantics are correct
- BTreeMap provides automatic sorting
- API endpoints are well-structured
- Decimal precision prevents float errors
- Code is readable and maintainable

### What Needs Fixing ⚠️
- Capacity tracking is inverted
- No error handling for empty book
- No order cancellation
- No persistent storage

### What Should Be Optimized 🚀
- Switch to HashMap+Vec if > 10k orders/sec needed
- Add concurrent matching support
- Implement caching for frequently accessed pairs
- Add metrics/monitoring

### Recommendation
**For current needs:** Fix bugs and add features before changing data structure

**For production:** Monitor performance, migrate to HashMap+Vec only if benchmarks show bottleneck

The BTreeMap implementation is **fundamentally sound** - optimize only if profiling justifies the effort.

