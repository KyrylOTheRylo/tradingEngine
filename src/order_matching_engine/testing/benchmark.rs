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

