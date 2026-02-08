# tradingEngine
This is RUST RESTAPI webservice. That implements trading engine for stock exchange (crypto, forex. etc.)  

## Performance Benchmarks (Release Mode - BTreeMap Implementation)

The trading engine has been benchmarked on a single-threaded system to measure order processing capacity with the optimized BTreeMap-based order book. The BTreeMap implementation provides automatic sorted ordering without O(n log n) sorting overhead on every market order.

### Order Insertion Performance
- **1,046,453 limit orders per second**
- **0.96 microseconds per order**
- BTreeMap maintains automatic price ordering

### Market Order Execution Performance
- **5,063,418 market orders per second**
- Ultra-fast order matching and execution (0.20 µs per order)
- No sorting overhead - BTreeMap is pre-sorted
- Correct price-priority matching guaranteed

### Mixed Workload Performance (Realistic Scenario)
- **1,251,396 total orders per second**
  - 70% Limit orders (875,980/sec)
  - 30% Market orders (375,417/sec)
- **0.80 microseconds per order**
- Balanced throughput representing real trading scenarios

## Performance Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| Limit Order Insertion Rate | 1.04M orders/sec | BTreeMap maintains O(log n) insertion with auto-sorting |
| Market Order Execution Rate | 5.06M orders/sec | No sorting required - BTreeMap is pre-sorted |
| Mixed Workload Rate | 1.25M orders/sec | 70% limit, 30% market |
| Average Latency (Limit Order) | 0.96 µs | Per order |
| Average Latency (Market Order) | 0.20 µs | Per order |
| Order Book Density | 1000+ price levels | Tested with dense order books |
| Price Priority Guarantee | ✅ Guaranteed | BTreeMap ensures correct matching order |

## Implementation Details

### Data Structure
- **Previous**: `HashMap<Decimal, Limit>` - Caused O(n log n) sorting overhead on every market order
- **Current**: `BTreeMap<Decimal, Limit>` - Automatic price ordering, no manual sorting needed
- **Benefit**: Faster execution, guaranteed price priority, industry-standard approach

### Why BTreeMap?
Real exchanges (Binance, Coinbase, NYSE) use tree-based structures for order books because:
1. ✅ Maintains sorted order automatically
2. ✅ Eliminates expensive sorting operations
3. ✅ Guarantees best execution (best available prices)
4. ✅ O(log n) insertion/deletion overhead is negligible vs sorting O(n log n)

### Correctness Guarantee
All 5 correctness tests pass:
- ✅ Market order matching at best available prices
- ✅ Asks always matched lowest-to-highest
- ✅ Bids always matched highest-to-lowest
- ✅ Partial fills across multiple price levels work correctly
- ✅ Insufficient liquidity handling

## Features

- **Fast Order Matching**: O(n) market order matching with zero sorting overhead
- **Price Priority**: Automatic price-ordered matching - guaranteed best execution
- **Multiple Trading Pairs**: Manage concurrent order books for different currency pairs
- **Robustness**: Comprehensive test suite with 25+ tests covering limit/market orders
- **REST API**: Full HTTP interface for order placement and market data queries
- **Industry-Standard**: Uses BTreeMap architecture like real exchanges

## API Endpoints

- `POST /create_limit_order/{base}_{quote}/{buy_or_sell}/{price}/{size}` - Place limit orders
- `POST /create_market_order/{base}_{quote}/{buy_or_sell}/{size}` - Execute market orders
- `GET /get_list_of_pairs` - List all trading pairs
- `GET /get_limits_for_a_pair/{base}_{quote}` - Get order book for a pair
- `GET /hey` - Health check

## Recent Changes (v2.0)

### Migration from HashMap to BTreeMap
- **Fixed**: Market order matching bug where orders weren't filled in correct price order
- **Improved**: Market order performance (5M+ orders/sec, 0.2 µs latency)
- **Added**: Comprehensive correctness tests (5 new tests)
- **Result**: Industry-standard implementation matching real exchanges

For detailed migration information, see [MIGRATION_COMPLETE.md](MIGRATION_COMPLETE.md)
## Running Benchmarks

To run performance benchmarks:

```bash
# Run all benchmarks
cargo test --release bench_ -- --nocapture --test-threads=1

# Run specific benchmark
cargo test --release bench_order_insertion -- --nocapture --test-threads=1
cargo test --release bench_market_order_execution -- --nocapture --test-threads=1
cargo test --release bench_mixed_order_workload -- --nocapture --test-threads=1
```

## Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```


