# tradingEngine
This is RUST RESTAPI webservice. That implements trading engine for stock exchange (crypto, forex. etc.)  

## Performance Benchmarks (Release Mode - HashMap Implementation)

The trading engine has been benchmarked on a single-threaded system to measure order processing capacity after migrating from BTreeMap to HashMap for O(1) insertion and lookup.

### Order Insertion Performance
- **25,030 limit orders per second**
- **39.95 microseconds per order**
- Optimized insertion with HashMap data structure

### Market Order Execution Performance
- **5,229,249 market orders per second**
- Extremely fast order matching and execution (0.19 µs per order)
- Efficient liquidity drainage

### Mixed Workload Performance (Realistic Scenario)
- **39,792 total orders per second**
  - 70% Limit orders (27,857/sec)
  - 30% Market orders (11,937/sec)
- **25.13 microseconds per order**
- Balanced throughput representing real trading scenarios

## Performance Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| Limit Order Insertion Rate | 25K orders/sec | Single-threaded, HashMap (no sorting overhead) |
| Market Order Execution Rate | 5.2M orders/sec | Against available liquidity |
| Mixed Workload Rate | 39.8K orders/sec | 70% limit, 30% market |
| Average Latency (Limit Order) | 39.95 µs | Per order |
| Average Latency (Market Order) | 0.19 µs | Per order |
| Order Book Density | 1000+ price levels | Tested with dense order books |
| Maximum Tested Orders | 1,890+ | Full robustness test suite |

## Features

- **Fast Order Matching**: Efficient matching engine with O(1) insertion via HashMap
- **Multiple Trading Pairs**: Manage concurrent order books for different currency pairs
- **Robustness**: Comprehensive test suite with 17+ tests covering limit/market orders
- **REST API**: Full HTTP interface for order placement and market data queries

## API Endpoints

- `POST /create_limit_order/{base}_{quote}/{buy_or_sell}/{price}/{size}` - Place limit orders
- `POST /create_market_order/{base}_{quote}/{buy_or_sell}/{size}` - Execute market orders
- `GET /get_list_of_pairs` - List all trading pairs
- `GET /get_limits_for_a_pair/{base}_{quote}` - Get order book for a pair
- `GET /hey` - Health check

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


