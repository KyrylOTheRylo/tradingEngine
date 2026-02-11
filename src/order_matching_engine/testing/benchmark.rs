// Benchmark test to measure order insertion performance
// Run with: cargo test --release bench_order_insertion -- --nocapture --test-threads=1

#[cfg(test)]
pub mod benchmark {
    use crate::order_matching_engine::engine::{EngineStats, MatchEngine, MarketId, TradingPair, PRICE_SCALE};
    use crate::order_matching_engine::orderbook::{BidOrAsk, Order, Tick};
    use std::hint::black_box;
    use std::time::Instant;

    fn env_usize(key: &str, default: usize) -> usize {
        std::env::var(key)
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(default)
    }

    fn print_throughput(
        title: &str,
        elapsed_secs: f64,
        commands: usize,
        market_orders: usize,
        stats: &EngineStats,
    ) {
        let commands_per_sec = commands as f64 / elapsed_secs.max(1e-12);
        let fills_per_sec = stats.fills_total as f64 / elapsed_secs.max(1e-12);
        let avg_fills_per_market = if market_orders == 0 {
            0.0
        } else {
            stats.fills_total as f64 / market_orders as f64
        };

        println!("\n========== {} ==========", title);
        println!("Elapsed: {:.6} seconds", elapsed_secs);
        println!("Commands: {}", commands);
        println!("Commands/sec: {:.0}", commands_per_sec);
        println!("Fills total: {}", stats.fills_total);
        println!("Fills/sec: {:.0}", fills_per_sec);
        println!("Avg fills per market order: {:.3}", avg_fills_per_market);
        println!("Levels crossed total: {}", stats.levels_crossed_total);
        println!("Resting orders consumed: {}", stats.resting_orders_consumed_total);
        println!("Total matched qty: {:.6}", stats.total_matched_qty);
        println!("============================================================\n");
    }

    fn base_tick() -> Tick {
        100 * PRICE_SCALE
    }

    fn spread_tick() -> Tick {
        PRICE_SCALE
    }

    #[test]
    fn bench_order_insertion_rate() {
        let mut engine: MatchEngine = MatchEngine::new();
        let market_id: MarketId = engine.add_new_market(TradingPair::new("btc".to_string(), "usd".to_string()));

        let iterations = env_usize("BENCH_INSERT_ITERS", 500_000);
        let levels = (iterations / 2) + 1;
        let base = base_tick();
        let spread = spread_tick();

        let mut bid_ticks: Vec<Tick> = Vec::with_capacity(levels);
        let mut ask_ticks: Vec<Tick> = Vec::with_capacity(levels);
        for i in 0..levels {
            let step = i as Tick * PRICE_SCALE;
            bid_ticks.push(base - spread - step);
            ask_ticks.push(base + spread + step);
        }

        engine.reset_stats();

        let start = Instant::now();
        let mut errors = 0usize;
        let mut bid_idx = 0usize;
        let mut ask_idx = 0usize;

        for i in 0..iterations {
            if i % 2 == 0 {
                let price = bid_ticks[bid_idx];
                bid_idx += 1;
                let order = Order::new(1.0, BidOrAsk::Bid);
                let result = engine.place_limit_order_raw_by_id_tick(market_id, black_box(price), order);
                black_box(&result);
                if result.is_err() {
                    errors += 1;
                }
            } else {
                let price = ask_ticks[ask_idx];
                ask_idx += 1;
                let order = Order::new(1.0, BidOrAsk::Ask);
                let result = engine.place_limit_order_raw_by_id_tick(market_id, black_box(price), order);
                black_box(&result);
                if result.is_err() {
                    errors += 1;
                }
            }
        }

        let elapsed = start.elapsed().as_secs_f64();
        let stats = engine.stats();

        print_throughput(
            "ORDER INSERTION BENCHMARK RESULTS",
            elapsed,
            iterations,
            0,
            &stats,
        );

        assert!(errors == 0, "Limit order placement errors: {}", errors);
    }

    #[test]
    fn bench_market_order_execution_rate() {
        let mut engine: MatchEngine = MatchEngine::new();
        let market_id: MarketId = engine.add_new_market(TradingPair::new("btc".to_string(), "usd".to_string()));

        let iterations = env_usize("BENCH_MARKET_ITERS", 10_000);
        let levels = 100usize;
        let orders_per_level = (iterations / levels) + 1;
        let limit_size = 1.0_f64;
        let market_size = 1.0_f64;
        let base = base_tick();

        // Add initial liquidity across 100 price levels
        for level in 0..levels {
            let price = base + (level as Tick * PRICE_SCALE);
            for _ in 0..orders_per_level {
                let order = Order::new(limit_size, BidOrAsk::Ask);
                let result = engine.place_limit_order_raw_by_id_tick(market_id, price, order);
                if result.is_err() {
                    panic!("Failed to prefill limit order: {:?}", result);
                }
            }
        }

        engine.reset_stats();

        let start = Instant::now();
        let mut errors = 0usize;
        for _ in 0..iterations {
            let mut market_order = Order::new(market_size, BidOrAsk::Bid);
            let result = engine.fill_market_order_raw_by_id(market_id, black_box(&mut market_order));
            black_box(&result);
            if result.is_err() {
                errors += 1;
            }
        }
        let elapsed = start.elapsed().as_secs_f64();
        let stats = engine.stats();

        print_throughput(
            "MARKET ORDER EXECUTION BENCHMARK RESULTS",
            elapsed,
            iterations,
            iterations,
            &stats,
        );

        assert!(errors == 0, "Market order execution errors: {}", errors);
    }

    #[test]
    fn bench_large_market_sweep_against_dense_single_level() {
        let mut engine: MatchEngine = MatchEngine::new();
        let market_id: MarketId = engine.add_new_market(TradingPair::new("btc".to_string(), "usd".to_string()));

        let limit_price = base_tick();
        let small_limit_size = 0.1_f64;
        let market_order_size = 20.0_f64;
        let market_order_target = env_usize("BENCH_DENSE_SWEEP_MARKETS", 10);
        let small_limit_count =
            (((market_order_target as f64 * market_order_size) / small_limit_size).ceil() as usize) + 1;

        // Many small sell orders at the same price level.
        for _ in 0..small_limit_count {
            let order = Order::new(small_limit_size, BidOrAsk::Ask);
            let result = engine.place_limit_order_raw_by_id_tick(market_id, limit_price, order);
            if result.is_err() {
                panic!("Failed to prefill dense level: {:?}", result);
            }
        }

        engine.reset_stats();

        let mut latencies = Vec::with_capacity(market_order_target);
        let start_total = Instant::now();
        let mut errors = 0usize;
        for _ in 0..market_order_target {
            let mut market_order = Order::new(market_order_size, BidOrAsk::Bid);
            let start = Instant::now();
            let result = engine.fill_market_order_raw_by_id(market_id, &mut market_order);
            let elapsed = start.elapsed().as_secs_f64();
            latencies.push(elapsed);
            if result.is_err() {
                errors += 1;
            }
        }
        let elapsed_total = start_total.elapsed().as_secs_f64();
        let stats = engine.stats();

        let worst_latency = latencies
            .iter()
            .cloned()
            .fold(0.0_f64, f64::max);
        let avg_latency = if latencies.is_empty() {
            0.0
        } else {
            latencies.iter().sum::<f64>() / latencies.len() as f64
        };

        println!("\n========== LARGE MARKET SWEEP BENCHMARK RESULTS ==========");
        println!("Small limit orders at one level: {}", small_limit_count);
        println!("Small limit size: {:.4}", small_limit_size);
        println!("Market order size: {:.4}", market_order_size);
        println!("Market orders executed: {}", market_order_target);
        println!("Elapsed: {:.6} seconds", elapsed_total);
        println!("Worst latency per market order: {:.6} s", worst_latency);
        println!("Avg latency per market order: {:.6} s", avg_latency);
        println!("=========================================================\n");

        print_throughput(
            "DENSE SINGLE-LEVEL SWEEP THROUGHPUT",
            elapsed_total,
            market_order_target,
            market_order_target,
            &stats,
        );

        assert!(errors == 0, "Market sweep errors: {}", errors);
    }

    #[test]
    fn bench_heavy_market_orders_against_small_limits_fixed() {
        let mut engine: MatchEngine = MatchEngine::new();
        let market_id: MarketId = engine.add_new_market(TradingPair::new("btc".to_string(), "usd".to_string()));

        let limit_price = base_tick();
        let small_limit_size = 1.0_f64;
        let heavy_market_size = 50.0_f64;
        let limits_per_market = (heavy_market_size / small_limit_size) as usize;

        let prefill_markets: usize = std::env::var("HEAVY_PREFILL_MARKETS")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(1);
        let refill_markets: usize = std::env::var("HEAVY_REFILL_MARKETS")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(1);
        let market_orders = env_usize("BENCH_HEAVY_MARKET_ITERS", 1_000);

        let small_limit_count = limits_per_market * prefill_markets;
        let limits_after_market = limits_per_market * refill_markets;

        for _ in 0..small_limit_count {
            let order = Order::new(small_limit_size, BidOrAsk::Ask);
            let _ = engine.place_limit_order_raw_by_id_tick(market_id, limit_price, order);
        }

        engine.reset_stats();

        let start = Instant::now();
        let mut market_order_count = 0usize;
        let mut refill_limit_count = 0usize;
        let mut errors = 0usize;

        for _ in 0..market_orders {
            let mut market_order = Order::new(heavy_market_size, BidOrAsk::Bid);
            let result = engine.fill_market_order_raw_by_id(market_id, &mut market_order);
            if result.is_err() || !market_order.is_filled() {
                errors += 1;
                break;
            }
            market_order_count += 1;

            for _ in 0..limits_after_market {
                let order = Order::new(small_limit_size, BidOrAsk::Ask);
                let _ = engine.place_limit_order_raw_by_id_tick(market_id, limit_price, order);
            }
            refill_limit_count += limits_after_market;
        }

        let elapsed = start.elapsed().as_secs_f64();
        let stats = engine.stats();
        let total_commands = market_order_count + refill_limit_count;

        println!("\n========== HEAVY MARKET ORDER FIXED BENCHMARK RESULTS ==========");
        println!("Small limit orders at one level: {}", small_limit_count);
        println!("Refill limit orders: {}", refill_limit_count);
        println!("Limits per market order: {}", limits_per_market);
        println!("Prefill markets: {}", prefill_markets);
        println!("Refill markets: {}", refill_markets);
        println!("Market orders executed: {}", market_order_count);
        println!("Total commands executed: {}", total_commands);
        println!("Elapsed: {:.6} seconds", elapsed);
        println!("===============================================================\n");

        print_throughput(
            "HEAVY MARKET ORDER THROUGHPUT",
            elapsed,
            total_commands,
            market_order_count,
            &stats,
        );

        assert!(errors == 0, "Heavy market order errors: {}", errors);
    }

    #[test]
    fn bench_mixed_order_workload() {
        let mut engine: MatchEngine = MatchEngine::new();
        let market_id: MarketId = engine.add_new_market(TradingPair::new("btc".to_string(), "usd".to_string()));

        let iterations = env_usize("BENCH_MIXED_ITERS", 200_000);
        let base = base_tick();
        let spread = spread_tick();
        let levels = (iterations / 2) + 1;

        let mut bid_ticks: Vec<Tick> = Vec::with_capacity(levels);
        let mut ask_ticks: Vec<Tick> = Vec::with_capacity(levels);
        for i in 0..levels {
            let step = i as Tick * PRICE_SCALE;
            bid_ticks.push(base - spread - step);
            ask_ticks.push(base + spread + step);
        }

        // Prefill some liquidity on both sides to avoid early rejections
        for i in 0..1_000usize {
            let price = base + spread + (i as Tick % 10) * PRICE_SCALE;
            let _ = engine.place_limit_order_raw_by_id_tick(market_id, price, Order::new(1.0, BidOrAsk::Ask));
            let price = base - spread - (i as Tick % 10) * PRICE_SCALE;
            let _ = engine.place_limit_order_raw_by_id_tick(market_id, price, Order::new(1.0, BidOrAsk::Bid));
        }

        engine.reset_stats();

        let start = Instant::now();
        let mut limit_order_count = 0usize;
        let mut market_order_count = 0usize;
        let mut bid_idx = 0usize;
        let mut ask_idx = 0usize;
        let mut errors = 0usize;

        for i in 0..iterations {
            let rand_val = i % 10;
            if rand_val < 7 {
                // 70% - Add limit order
                if limit_order_count % 2 == 0 {
                    let price = bid_ticks[bid_idx];
                    bid_idx += 1;
                    let order = Order::new(1.0, BidOrAsk::Bid);
                    let result = engine.place_limit_order_raw_by_id_tick(market_id, price, order);
                    if result.is_err() {
                        errors += 1;
                    }
                } else {
                    let price = ask_ticks[ask_idx];
                    ask_idx += 1;
                    let order = Order::new(1.0, BidOrAsk::Ask);
                    let result = engine.place_limit_order_raw_by_id_tick(market_id, price, order);
                    if result.is_err() {
                        errors += 1;
                    }
                }
                limit_order_count += 1;
            } else {
                // 30% - Execute market order
                let side = if market_order_count % 2 == 0 {
                    BidOrAsk::Bid
                } else {
                    BidOrAsk::Ask
                };
                let mut market_order = Order::new(0.5, side);
                let result = engine.fill_market_order_raw_by_id(market_id, &mut market_order);
                if result.is_err() {
                    errors += 1;
                }
                market_order_count += 1;
            }
        }

        let elapsed = start.elapsed().as_secs_f64();
        let total_commands = limit_order_count + market_order_count;
        let stats = engine.stats();

        println!("\n========== MIXED ORDER WORKLOAD BENCHMARK RESULTS ==========");
        println!("Total commands: {}", total_commands);
        println!("  - Limit orders: {} ({:.1}%)", limit_order_count, (limit_order_count as f64 / total_commands as f64) * 100.0);
        println!("  - Market orders: {} ({:.1}%)", market_order_count, (market_order_count as f64 / total_commands as f64) * 100.0);
        println!("Elapsed: {:.6} seconds", elapsed);
        println!("===========================================================\n");

        print_throughput(
            "MIXED WORKLOAD THROUGHPUT",
            elapsed,
            total_commands,
            market_order_count,
            &stats,
        );

        assert!(errors == 0, "Mixed workload errors: {}", errors);
    }

    #[test]
    fn bench_tail_latency_single_level_sweep() {
        let mut engine: MatchEngine = MatchEngine::new();
        let market_id: MarketId = engine.add_new_market(TradingPair::new("btc".to_string(), "usd".to_string()));

        let limit_price = base_tick();
        let small_limit_size = 1.0_f64;
        let small_limit_count = env_usize("BENCH_TAIL_SWEEP_LIMITS", 100_000);
        let market_order_size = small_limit_count as f64 * small_limit_size;

        for _ in 0..small_limit_count {
            let order = Order::new(small_limit_size, BidOrAsk::Ask);
            let result = engine.place_limit_order_raw_by_id_tick(market_id, limit_price, order);
            if result.is_err() {
                panic!("Failed to prefill tail sweep: {:?}", result);
            }
        }

        engine.reset_stats();

        let mut market_order = Order::new(market_order_size, BidOrAsk::Bid);
        let start = Instant::now();
        let result = engine.fill_market_order_raw_by_id(market_id, &mut market_order);
        let elapsed = start.elapsed().as_secs_f64();
        let stats = engine.stats();

        println!("\n========== TAIL LATENCY SINGLE-LEVEL SWEEP ==========");
        println!("Small limit orders: {}", small_limit_count);
        println!("Small limit size: {:.6}", small_limit_size);
        println!("Market order size: {:.6}", market_order_size);
        println!("Elapsed: {:.6} seconds", elapsed);
        println!("Fills total: {}", stats.fills_total);
        println!("Fills/sec: {:.0}", stats.fills_total as f64 / elapsed.max(1e-12));
        println!("=====================================================\n");

        assert!(result.is_ok(), "Tail sweep failed: {:?}", result);
    }
}
