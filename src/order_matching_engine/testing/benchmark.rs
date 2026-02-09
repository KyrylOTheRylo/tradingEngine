// Benchmark test to measure order insertion performance
// Run with: cargo test --release bench_order_insertion -- --nocapture --test-threads=1

#[cfg(test)]
pub mod benchmark {
    use crate::order_matching_engine::orderbook::{Order, BidOrAsk};
    use crate::order_matching_engine::engine::{TradingPair, MatchEngine};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::time::Instant;

    #[test]
    fn bench_order_insertion_rate() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        let start = Instant::now();
        let mut order_count = 0;

        // Insert as many orders as possible in 1 second
        while start.elapsed().as_secs_f64() < 1.0 {
            // Alternate between buy and sell orders
            if order_count % 2 == 0 {
                let order = Order::new(1.0, BidOrAsk::Bid);
                let price = dec!(100) - Decimal::from(order_count / 2);
                let _ = engine.place_limit_order(btc_usd.clone(), price, order);
            } else {
                let order = Order::new(1.0, BidOrAsk::Ask);
                let price = dec!(100) + Decimal::from(order_count / 2);
                let _ = engine.place_limit_order(btc_usd.clone(), price, order);
            }
            order_count += 1;
        }

        let elapsed = start.elapsed();
        let orders_per_second = order_count as f64 / elapsed.as_secs_f64();

        println!("\n========== ORDER INSERTION BENCHMARK RESULTS ==========");
        println!("Total orders inserted: {}", order_count);
        println!("Time elapsed: {:.3} seconds", elapsed.as_secs_f64());
        println!("Orders per second: {:.0}", orders_per_second);
        println!("Microseconds per order: {:.2}", 1_000_000.0 / orders_per_second);
        println!("======================================================\n");

        // Assert we can handle at least 1000 orders per second
        assert!(orders_per_second > 1000.0, "Should handle at least 1000 orders/sec");
    }

    #[test]
    fn bench_market_order_execution_rate() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        // Add initial liquidity (large sell wall)
        for i in 0..100 {
            let order = Order::new(100.0, BidOrAsk::Ask);
            let _ = engine.place_limit_order(btc_usd.clone(), dec!(100) + Decimal::from(i), order);
        }

        let start = Instant::now();
        let mut order_count = 0;

        // Execute market orders as fast as possible
        while start.elapsed().as_secs_f64() < 1.0 {
            let mut market_order = Order::new(5.0, BidOrAsk::Bid);
            let _ = engine.fill_market_order(btc_usd.clone(), &mut market_order);
            order_count += 1;
        }

        let elapsed = start.elapsed();
        let orders_per_second = order_count as f64 / elapsed.as_secs_f64();

        println!("\n========== MARKET ORDER EXECUTION BENCHMARK RESULTS ==========");
        println!("Total market orders executed: {}", order_count);
        println!("Time elapsed: {:.3} seconds", elapsed.as_secs_f64());
        println!("Market orders per second: {:.0}", orders_per_second);
        println!("Microseconds per market order: {:.2}", 1_000_000.0 / orders_per_second);
        println!("==============================================================\n");

        // Assert we can handle at least 1000 market orders per second
        assert!(orders_per_second > 1000.0, "Should handle at least 1000 market orders/sec");
    }

    #[test]
    fn bench_large_market_sweep_against_dense_single_level() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        let limit_price = dec!(100);
        let small_limit_size = 0.1_f64;
        let market_order_size = 20.0_f64;
        let market_order_target: usize = 10;
        let max_duration_secs = 5.0_f64;
        let small_limit_count =
            ((market_order_target as f64 * market_order_size) / small_limit_size) as usize;

        // Many small sell orders at the same price level.
        for _ in 0..small_limit_count {
            let order = Order::new(small_limit_size, BidOrAsk::Ask);
            let _ = engine.place_limit_order(btc_usd.clone(), limit_price, order);
        }

        let total_limit_volume = small_limit_size * small_limit_count as f64;
        let max_market_orders = (total_limit_volume / market_order_size) as usize;
        assert!(
            max_market_orders > 0,
            "Benchmark setup needs enough limit volume to fill market orders"
        );

        let start = Instant::now();
        let mut market_order_count = 0;
        while start.elapsed().as_secs_f64() < max_duration_secs
            && market_order_count < max_market_orders
            && market_order_count < market_order_target
        {
            let mut market_order = Order::new(market_order_size, BidOrAsk::Bid);
            let _ = engine.fill_market_order(btc_usd.clone(), &mut market_order);
            market_order_count += 1;
        }

        let elapsed = start.elapsed();
        let elapsed_secs = elapsed.as_secs_f64().max(1e-9);
        let orders_per_second = market_order_count as f64 / elapsed_secs;

        println!("\n========== LARGE MARKET SWEEP BENCHMARK RESULTS ==========");
        println!("Small limit orders at one level: {}", small_limit_count);
        println!("Small limit size: {:.4}", small_limit_size);
        println!("Market order size: {}", market_order_size);
        println!("Market orders executed: {}", market_order_count);
        println!("Time elapsed: {:.3} seconds", elapsed_secs);
        println!("Market orders per second: {:.0}", orders_per_second);
        println!("Microseconds per market order: {:.2}", 1_000_000.0 / orders_per_second);
        println!("=========================================================\n");

        // Smoke check: completes quickly and processes some large market orders.
        assert!(
            elapsed_secs <= max_duration_secs + 0.1,
            "Benchmark should complete within the time budget"
        );
        assert_eq!(
            market_order_count, market_order_target,
            "Should execute the planned number of market orders"
        );
    }

    #[test]
    fn bench_heavy_market_orders_against_small_limits_1s() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        let limit_price = dec!(100);
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
        let small_limit_count = limits_per_market * prefill_markets;
        let limits_after_market = limits_per_market * refill_markets;

        assert!(
            (heavy_market_size / small_limit_size - limits_per_market as f64).abs() < 1e-9,
            "heavy_market_size should be an exact multiple of small_limit_size"
        );
        assert!(prefill_markets > 0, "Need at least 1 prefill market buffer");
        assert!(refill_markets > 0, "Need at least 1 refill market buffer");

        for _ in 0..small_limit_count {
            let order = Order::new(small_limit_size, BidOrAsk::Ask);
            let _ = engine.place_limit_order(btc_usd.clone(), limit_price, order);
        }

        let start = Instant::now();
        let mut exhausted_at: Option<f64> = None;
        let mut market_order_count = 0;
        let mut refill_limit_count = 0;
        while start.elapsed().as_secs_f64() < 1.0 {
            let mut market_order = Order::new(heavy_market_size, BidOrAsk::Bid);
            let _ = engine.fill_market_order(btc_usd.clone(), &mut market_order);
            if market_order.is_filled() {
                market_order_count += 1;
                for _ in 0..limits_after_market {
                    let order = Order::new(small_limit_size, BidOrAsk::Ask);
                    let _ = engine.place_limit_order(btc_usd.clone(), limit_price, order);
                }
                refill_limit_count += limits_after_market;
            } else {
                exhausted_at = Some(start.elapsed().as_secs_f64());
                break;
            }
        }

        let elapsed = start.elapsed();
        let exec_elapsed = exhausted_at.unwrap_or_else(|| elapsed.as_secs_f64());
        let orders_per_second = market_order_count as f64 / exec_elapsed.max(1e-9);

        if let Some(exhausted_secs) = exhausted_at {
            let remaining = if exhausted_secs < 1.0 {
                1.0 - exhausted_secs
            } else {
                0.0
            };
            if remaining > 0.0 {
                std::thread::sleep(std::time::Duration::from_secs_f64(remaining));
            }
        }

        let total_limit_orders = small_limit_count + refill_limit_count;
        let total_orders = total_limit_orders + market_order_count;

        println!("\n========== HEAVY MARKET ORDER 1s BENCHMARK RESULTS ==========");
        println!("Small limit orders at one level: {}", small_limit_count);
        println!("Small limits added after market orders: {}", refill_limit_count);
        println!("Total limit orders placed: {}", total_limit_orders);
        println!("Limits per market order: {}", limits_per_market);
        println!("Prefill markets: {}", prefill_markets);
        println!("Refill markets: {}", refill_markets);
        println!("Small limit size: {:.4}", small_limit_size);
        println!("Heavy market order size: {}", heavy_market_size);
        println!("Market orders executed: {}", market_order_count);
        println!("Total orders processed: {}", total_orders);
        println!("Execution time: {:.3} seconds", exec_elapsed);
        println!("Market orders per second: {:.0}", orders_per_second);
        println!("Total orders per second: {:.0}", total_orders as f64 / elapsed.as_secs_f64());
        println!("Microseconds per market order: {:.2}", 1_000_000.0 / orders_per_second);
        if let Some(exhausted_secs) = exhausted_at {
            println!("Liquidity exhausted at {:.3} seconds", exhausted_secs);
            println!("Increase HEAVY_PREFILL_MARKETS or HEAVY_REFILL_MARKETS for a full 1s run");
        }
        println!("============================================================\n");

        assert!(market_order_count > 0, "Should execute at least one market order");
    }

    #[test]
    fn bench_mixed_order_workload() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        let start = Instant::now();
        let mut limit_order_count = 0;
        let mut market_order_count = 0;

        // Workload: 70% limit orders, 30% market orders
        while start.elapsed().as_secs_f64() < 1.0 {
            let rand_val = (limit_order_count + market_order_count) % 10;

            if rand_val < 7 {
                // 70% - Add limit order
                if limit_order_count % 2 == 0 {
                    let order = Order::new(1.0, BidOrAsk::Bid);
                    let price = dec!(100) - Decimal::from(limit_order_count / 2);
                    let _ = engine.place_limit_order(btc_usd.clone(), price, order);
                } else {
                    let order = Order::new(1.0, BidOrAsk::Ask);
                    let price = dec!(100) + Decimal::from(limit_order_count / 2);
                    let _ = engine.place_limit_order(btc_usd.clone(), price, order);
                }
                limit_order_count += 1;
            } else {
                // 30% - Execute market order
                let mut market_order = Order::new(0.5, BidOrAsk::Bid);
                let _ = engine.fill_market_order(btc_usd.clone(), &mut market_order);
                market_order_count += 1;
            }
        }

        let elapsed = start.elapsed();
        let total_orders = limit_order_count + market_order_count;
        let total_per_second = total_orders as f64 / elapsed.as_secs_f64();

        println!("\n========== MIXED ORDER WORKLOAD BENCHMARK RESULTS ==========");
        println!("Total orders processed: {}", total_orders);
        println!("  - Limit orders: {} ({:.1}%)", limit_order_count, (limit_order_count as f64 / total_orders as f64) * 100.0);
        println!("  - Market orders: {} ({:.1}%)", market_order_count, (market_order_count as f64 / total_orders as f64) * 100.0);
        println!("Time elapsed: {:.3} seconds", elapsed.as_secs_f64());
        println!("Total orders per second: {:.0}", total_per_second);
        println!("Microseconds per order: {:.2}", 1_000_000.0 / total_per_second);
        println!("=============================================================\n");

        // Assert we can handle at least 1000 orders per second in mixed workload
        assert!(total_per_second > 1000.0, "Should handle at least 1000 orders/sec in mixed workload");
    }
}
