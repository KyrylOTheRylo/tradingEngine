# tradingEngine
This is RUST RESTAPI webservice. That implements trading engine for stock exchange (crypto, forex. etc.)  

## Performance Benchmarks (Release Mode)

The trading engine has been benchmarked on a single-threaded system to measure order processing capacity:

### Order Insertion Performance
- **25,327 limit orders per second**
- **39.48 microseconds per order**
- Sustainable throughput with stable latency

### Market Order Execution Performance
- **5,256,438 market orders per second**
- Extremely fast order matching and execution
- Efficient liquidity drainage

### Mixed Workload Performance (Realistic Scenario)
- **38,237 total orders per second**
  - 70% Limit orders (26,768/sec)
  - 30% Market orders (11,469/sec)
- **26.15 microseconds per order**
- Balanced throughput representing real trading scenarios

## Performance Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| Limit Order Insertion Rate | 25K orders/sec | Single-threaded |
| Market Order Execution Rate | 5.2M orders/sec | Against available liquidity |
| Mixed Workload Rate | 38K orders/sec | 70% limit, 30% market |
| Average Latency (Limit Order) | 39.48 µs | Per order |
| Average Latency (Market Order) | ~0.19 µs | Per order |
| Order Book Density | 1000+ price levels | Tested with dense order books |
| Maximum Tested Orders | 1,890+ | Full robustness test suite |

## Features

- **Fast Order Matching**: Efficient matching engine with O(1) price lookup via BTreeMap
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


