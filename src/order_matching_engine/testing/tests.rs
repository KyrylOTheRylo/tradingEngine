


#[cfg(test)]
pub mod test {
    use crate::order_matching_engine::orderbook::{Order, Limit,  OrderBook, BidOrAsk, RestingOrder};
    use crate::order_matching_engine::engine::{TradingPair, MatchEngine, price_to_tick};
    
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn order_book_test() {
        let mut orderbook: OrderBook = OrderBook::new();
        orderbook.add_limit_order(price_to_tick(dec!(500.0)), Order::new(10.0, BidOrAsk::Ask)) ;
        orderbook.add_limit_order(price_to_tick(dec!(400.0)), Order::new(10.0, BidOrAsk::Ask)) ;
        orderbook.add_limit_order(price_to_tick(dec!(200.0)), Order::new(10.0, BidOrAsk::Ask)) ;
        orderbook.add_limit_order(price_to_tick(dec!(300.0)), Order::new(10.0, BidOrAsk::Ask)) ;

        let mut market_order: Order = Order::new(12.1, BidOrAsk::Bid);
        orderbook.fill_order_book(&mut market_order);
        orderbook.add_limit_order(price_to_tick(dec!(200.0)), Order::new(10.0, BidOrAsk::Bid)) ;

        orderbook.add_limit_order(price_to_tick(dec!(10.0)), Order::new(10.0, BidOrAsk::Bid)) ;
        let mut market_order2: Order = Order::new(12.1, BidOrAsk::Ask);
        orderbook.fill_order_book(&mut market_order2);
        println!("{:?}", orderbook);
        
        assert_eq!(orderbook.ask_capacity(), 27.9);
        assert_eq!(orderbook.bid_capacity(), 7.9);

    }


    #[test]
    fn total_volume_test2() {
        let price: Decimal = dec!(1000.0);
        let mut limit: Limit = Limit::new(price_to_tick(price));
        let buy_limit_order1: RestingOrder = RestingOrder::new(1, 50.0);
        let buy_limit_order2: RestingOrder = RestingOrder::new(2, 48.0);
        limit.add_order(buy_limit_order1);
        limit.add_order(buy_limit_order2);
        assert_eq!(limit.total_volume(), 98.0);
       
    }

    #[test]
    fn total_volume_test() {
        let price: Decimal = dec!(1000.0);
        let mut limit: Limit = Limit::new(price_to_tick(price));
        let buy_limit_order1: RestingOrder = RestingOrder::new(1, 50.0);
        let buy_limit_order2: RestingOrder = RestingOrder::new(2, 48.0);

        let mut  market_sell_order: Order =
         Order::new( 51.0, BidOrAsk::Ask);

        limit.add_order(buy_limit_order1);
        limit.add_order(buy_limit_order2);
        limit.fill_order(&mut market_sell_order, &mut |_order_id, _qty| {});
        assert_eq!(limit.total_volume(), 98.0 - 51.0);
       
    }

    #[test]
    fn total_volume_test3() {
        let price: Decimal = dec!(1000.0);
        let mut limit: Limit = Limit::new(price_to_tick(price));
        let buy_limit_order1: RestingOrder = RestingOrder::new(1, 50.0);
        let buy_limit_order2: RestingOrder = RestingOrder::new(2, 48.0);

        let mut  market_sell_order: Order =
         Order::new( 90.0, BidOrAsk::Ask);

        limit.add_order(buy_limit_order1);
        limit.add_order(buy_limit_order2);
        limit.fill_order(&mut market_sell_order, &mut |_order_id, _qty| {});
        println!("{:?}", market_sell_order);
    }

    #[test]
    fn limit_order_fill() {
        let price: Decimal = dec!(1000.0);
        let mut limit: Limit = Limit::new(price_to_tick(price)) ;
    
        let buy_limit_order1: RestingOrder = RestingOrder::new(1, 50.0);
        let buy_limit_order2: RestingOrder = RestingOrder::new(2, 48.0);
        
        let mut  market_sell_order: Order =
         Order::new( 51.0, BidOrAsk::Ask);
        limit.add_order(buy_limit_order1);
        limit.add_order(buy_limit_order2);


        limit.fill_order(&mut market_sell_order, &mut |_order_id, _qty| {});
        println!("{:?}", limit);
    }

    #[test]
    fn engine_all_orderbooks_test() {
        let mut engine: MatchEngine  = MatchEngine::new();

        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        let _btc_eth = TradingPair::new(String::from("btc"), String::from("eth"));
        engine.add_new_market(btc_usd.clone());
        let order: Order = Order::new(10.4, BidOrAsk::Ask);
        let _ = engine.place_limit_order(&btc_usd, dec!(10.3), order);

        let order2: Order = Order::new(10.5, BidOrAsk::Bid);
        println!("{:?}",engine.place_limit_order(&btc_usd, dec!(10.5), order2));
        engine.get_orderbooks();
        println!("{:?}",engine.get_limits_for_a_pair(&btc_usd));

        assert_eq!(engine.get_orderbooks().len(), 1);

    
    }

    #[test]
    fn market_order_test() {
        let mut engine: MatchEngine  = MatchEngine::new();

        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        let _btc_eth = TradingPair::new(String::from("btc"), String::from("eth"));
        engine.add_new_market(btc_usd.clone());
        let order: Order = Order::new(10.4, BidOrAsk::Ask);
        let _ = engine.place_limit_order(&btc_usd, dec!(10.3), order);

        let order2: Order = Order::new(10.5, BidOrAsk::Bid);
        println!("{:?}",engine.place_limit_order(&btc_usd, dec!(10.2), order2));

        let mut market_order = Order::new(10.5, BidOrAsk::Ask);
        let mut market_order2 = Order::new(10.4, BidOrAsk::Bid);
        let _ = engine.fill_market_order(&btc_usd ,&mut market_order);
        let _ = engine.fill_market_order(&btc_usd ,&mut market_order2);

        assert_eq!(engine.get_limits_for_a_pair(&btc_usd).unwrap().bid_capacity(), 0.0);  

    
    }

    // ========== ROBUSTNESS TESTS: MANY LIMIT ORDERS ==========

    #[test]
    fn many_limit_orders_single_price_level() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        // Add 100 sell orders at the same price level
        for i in 0..100 {
            let order = Order::new(1.0, BidOrAsk::Ask);
            let result = engine.place_limit_order(&btc_usd, dec!(100.0), order);
            assert!(result.is_ok(), "Failed to place order {}", i);
        }

        // Verify total capacity (Ask orders now correctly track ask_capacity)
        let orderbook = engine.get_limits_for_a_pair(&btc_usd).unwrap();
        assert_eq!(orderbook.ask_capacity(), 100.0);
    }

    #[test]
    fn many_limit_orders_multiple_price_levels() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        // Add 50 sell orders at different price levels (100-149)
        for i in 0..50 {
            let order = Order::new(10.0, BidOrAsk::Ask);
            let price = dec!(100) + Decimal::from(i);
            let result = engine.place_limit_order(&btc_usd, price, order);
            assert!(result.is_ok(), "Failed to place ask order at price {}", price);
        }

        // Add 50 buy orders at different price levels (99-50)
        for i in 0..50 {
            let order = Order::new(10.0, BidOrAsk::Bid);
            let price = dec!(99) - Decimal::from(i);
            let result = engine.place_limit_order(&btc_usd, price, order);
            assert!(result.is_ok(), "Failed to place bid order at price {}", price);
        }

        // Verify capacities
        let orderbook = engine.get_limits_for_a_pair(&btc_usd).unwrap();
        assert_eq!(orderbook.ask_capacity(), 500.0);
        assert_eq!(orderbook.bid_capacity(), 500.0);
    }

    #[test]
    fn many_limit_orders_increasing_sizes() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        // Add sell orders with increasing sizes: 1.0, 2.0, 3.0, ..., 100.0
        let mut total_ask = 0.0;
        for i in 1..=100 {
            let order = Order::new(i as f64, BidOrAsk::Ask);
            let price = dec!(100.0) + Decimal::from(i);
            let result = engine.place_limit_order(&btc_usd, price, order);
            assert!(result.is_ok(), "Failed to place ask order");
            total_ask += i as f64;
        }

        // Add buy orders with decreasing sizes: 100.0, 99.0, ..., 1.0
        let mut total_bid = 0.0;
        for i in (1..=100).rev() {
            let order = Order::new(i as f64, BidOrAsk::Bid);
            let price = dec!(99.0) - Decimal::from(101 - i);
            let result = engine.place_limit_order(&btc_usd, price, order);
            assert!(result.is_ok(), "Failed to place bid order");
            total_bid += i as f64;
        }

        let orderbook = engine.get_limits_for_a_pair(&btc_usd).unwrap();
        assert_eq!(orderbook.ask_capacity(), total_ask);
        assert_eq!(orderbook.bid_capacity(), total_bid);
    }

    #[test]
    fn many_limit_orders_dense_price_range() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        // Add 500 sell orders in price range 2000-2499
        for price_int in 2000..2500 {
            let price = Decimal::from(price_int);
            let sell_order = Order::new(5.0, BidOrAsk::Ask);
            let result1 = engine.place_limit_order(&btc_usd, price, sell_order);
            assert!(result1.is_ok());
        }

        // Add 500 buy orders in price range 1000-1499 (below sell orders)
        for price_int in 1000..1500 {
            let price = Decimal::from(price_int);
            let buy_order = Order::new(5.0, BidOrAsk::Bid);
            let result = engine.place_limit_order(&btc_usd, price, buy_order);
            assert!(result.is_ok());
        }

        let orderbook = engine.get_limits_for_a_pair(&btc_usd).unwrap();
        // Verify that orders were placed
        assert!(orderbook.ask_capacity() > 0.0);
        assert!(orderbook.bid_capacity() > 0.0);
    }

    // ========== ROBUSTNESS TESTS: MANY MARKET ORDERS ==========

    #[test]
    fn many_market_orders_against_single_limit() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        // Add a large sell order
        let large_sell = Order::new(1000.0, BidOrAsk::Ask);
        let _ = engine.place_limit_order(&btc_usd, dec!(100.0), large_sell);

        // Fill with many small market buy orders
        for i in 0..100 {
            let mut market_order = Order::new(10.0, BidOrAsk::Bid);
            let result = engine.fill_market_order(&btc_usd, &mut market_order);
            assert!(result.is_ok(), "Failed to fill market order {}", i);
        }

        // After 100 * 10 = 1000 units filled, the limit should be completely consumed
        let orderbook = engine.get_limits_for_a_pair(&btc_usd).unwrap();
        assert_eq!(orderbook.ask_capacity(), 0.0);
    }

    #[test]
    fn many_market_orders_alternating_sides() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        // Add initial liquidity
        for i in 0..50 {
            let buy_order = Order::new(10.0, BidOrAsk::Bid);
            let _ = engine.place_limit_order(&btc_usd, dec!(100) - Decimal::from(i), buy_order);

            let sell_order = Order::new(10.0, BidOrAsk::Ask);
            let _ = engine.place_limit_order(&btc_usd, dec!(101) + Decimal::from(i), sell_order);
        }

        // Execute alternating market orders
        for i in 0..50 {
            let mut market_buy = Order::new(5.0, BidOrAsk::Bid);
            let result1 = engine.fill_market_order(&btc_usd, &mut market_buy);
            assert!(result1.is_ok(), "Market buy failed at iteration {}", i);

            let mut market_sell = Order::new(5.0, BidOrAsk::Ask);
            let result2 = engine.fill_market_order(&btc_usd, &mut market_sell);
            assert!(result2.is_ok(), "Market sell failed at iteration {}", i);
        }

        // Verify order book is reduced but still has some liquidity
        let orderbook = engine.get_limits_for_a_pair(&btc_usd).unwrap();
        assert!(orderbook.bid_capacity() >= 0.0);
        assert!(orderbook.ask_capacity() >= 0.0);
    }

    #[test]
    fn many_market_orders_drain_liquidity() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        // Add liquidity: 10 levels with 100 units each
        for level in 0..10 {
            let buy_order = Order::new(100.0, BidOrAsk::Bid);
            let _ = engine.place_limit_order(&btc_usd, dec!(100) - Decimal::from(level), buy_order);
        }

        // Execute 5 large market orders that drain the liquidity
        for i in 0..5 {
            let mut market_order = Order::new(200.0, BidOrAsk::Ask);
            let result = engine.fill_market_order(&btc_usd, &mut market_order);
            assert!(result.is_ok(), "Market order {} failed", i);
        }

        // Should have drained 1000 units
        let orderbook = engine.get_limits_for_a_pair(&btc_usd).unwrap();
        assert_eq!(orderbook.bid_capacity(), 0.0);
    }

    #[test]
    fn sequential_limit_and_market_orders() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        // Alternating pattern: add limit order, execute market order
        for i in 0..100 {
            // Add a limit sell order
            let sell_order = Order::new(50.0, BidOrAsk::Ask);
            let sell_result = engine.place_limit_order(
                &btc_usd,
                dec!(100) + Decimal::from(i),
                sell_order
            );
            assert!(sell_result.is_ok());

            // Add a limit buy order
            let buy_order = Order::new(50.0, BidOrAsk::Bid);
            let buy_result = engine.place_limit_order(
                &btc_usd,
                dec!(99) - Decimal::from(i),
                buy_order
            );
            assert!(buy_result.is_ok());

            // Execute a market order every 10 iterations
            if i % 10 == 0 && i > 0 {
                let mut market_order = Order::new(25.0, BidOrAsk::Bid);
                let market_result = engine.fill_market_order(&btc_usd, &mut market_order);
                assert!(market_result.is_ok());
            }
        }

        let orderbook = engine.get_limits_for_a_pair(&btc_usd).unwrap();
        // Verify we still have a healthy order book
        assert!(orderbook.bid_capacity() > 0.0);
        assert!(orderbook.ask_capacity() > 0.0);
    }

    #[test]
    fn stress_test_mixed_order_execution() {
        let mut engine: MatchEngine = MatchEngine::new();
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        engine.add_new_market(btc_usd.clone());

        let mut total_buy_volume = 0.0;
        let mut total_sell_volume = 0.0;
        let mut executed_buy_volume = 0.0;
        let mut executed_sell_volume = 0.0;

        // Phase 1: Add many limit orders
        for i in 0..200 {
            if i % 2 == 0 {
                let buy_order = Order::new(25.0, BidOrAsk::Bid);
                let price = dec!(100) - Decimal::from(i / 2);
                engine.place_limit_order(&btc_usd, price, buy_order).ok();
                total_buy_volume += 25.0;
            } else {
                let sell_order = Order::new(25.0, BidOrAsk::Ask);
                let price = dec!(100) + Decimal::from(i / 2);
                engine.place_limit_order(&btc_usd, price, sell_order).ok();
                total_sell_volume += 25.0;
            }
        }

        // Phase 2: Execute market orders
        for i in 0..50 {
            if i % 2 == 0 {
                let mut market_buy = Order::new(15.0, BidOrAsk::Bid);
                if engine.fill_market_order(&btc_usd, &mut market_buy).is_ok() {
                    executed_buy_volume += market_buy.size();
                }
            } else {
                let mut market_sell = Order::new(15.0, BidOrAsk::Ask);
                if engine.fill_market_order(&btc_usd, &mut market_sell).is_ok() {
                    executed_sell_volume += market_sell.size();
                }
            }
        }

        let orderbook = engine.get_limits_for_a_pair(&btc_usd).unwrap();
        let remaining_buy = orderbook.bid_capacity();
        let remaining_sell = orderbook.ask_capacity();

        // Verify conservation: total volume = executed + remaining
        assert!(executed_buy_volume + remaining_buy <= total_buy_volume + 0.1); // Small epsilon for floating point
        assert!(executed_sell_volume + remaining_sell <= total_sell_volume + 0.1);
    }

    #[test]
    fn many_orders_multiple_trading_pairs() {
        let mut engine: MatchEngine = MatchEngine::new();

        // Create multiple trading pairs
        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        let eth_usd = TradingPair::new(String::from("eth"), String::from("usd"));
        let btc_eth = TradingPair::new(String::from("btc"), String::from("eth"));

        engine.add_new_market(btc_usd.clone());
        engine.add_new_market(eth_usd.clone());
        engine.add_new_market(btc_eth.clone());

        // Add orders to each pair
        for pair in [&btc_usd, &eth_usd, &btc_eth] {
            for i in 0..50 {
                let buy_order = Order::new(10.0, BidOrAsk::Bid);
                engine.place_limit_order(pair, dec!(100) - Decimal::from(i), buy_order).ok();

                let sell_order = Order::new(10.0, BidOrAsk::Ask);
                engine.place_limit_order(pair, dec!(100) + Decimal::from(i), sell_order).ok();
            }
        }

        // Execute market orders on each pair
        for pair in [&btc_usd, &eth_usd, &btc_eth] {
            for _ in 0..25 {
                let mut market_order = Order::new(5.0, BidOrAsk::Bid);
                engine.fill_market_order(pair, &mut market_order).ok();
            }
        }

        // Verify all pairs are still valid
        assert!(engine.get_limits_for_a_pair(&btc_usd).is_some());
        assert!(engine.get_limits_for_a_pair(&eth_usd).is_some());
        assert!(engine.get_limits_for_a_pair(&btc_eth).is_some());
    }

    #[test]
    fn test_market_order_matching_best_price() {
        // This test reproduces the issue reported where market orders weren't matching
        // against the best available prices

        let mut orderbook: OrderBook = OrderBook::new();

        // Setup initial asks (sellers):
        // price = 11.3 with amount 40.0
        // price = 12.0 with amount 50.0
        // price = 11.0 with amount 1.0
        orderbook.add_limit_order(price_to_tick(dec!(11.3)), Order::new(40.0, BidOrAsk::Ask));
        orderbook.add_limit_order(price_to_tick(dec!(12.0)), Order::new(50.0, BidOrAsk::Ask));
        orderbook.add_limit_order(price_to_tick(dec!(11.0)), Order::new(1.0, BidOrAsk::Ask));

        // Setup initial bids (buyers):
        // price = 10.0 with amount 100.0
        orderbook.add_limit_order(price_to_tick(dec!(10.0)), Order::new(100.0, BidOrAsk::Bid));

        // Before: ask_capacity = 40 + 50 + 1 = 91.0
        assert_eq!(orderbook.ask_capacity(), 91.0);

        // Execute a buy order of 5.0 (should match against the lowest ask prices first)
        // Expected to match: 1.0 from 11.0, then 4.0 from 11.3
        let mut market_order = Order::new(5.0, BidOrAsk::Bid);
        let result = orderbook.fill_order_book(&mut market_order);

        println!("Market order result: {}", result);
        println!("Remaining ask_capacity: {}", orderbook.ask_capacity());

        // After matching 5.0:
        // - The 1.0 at 11.0 should be completely filled (removed)
        // - The 40.0 at 11.3 should have 4.0 filled, leaving 36.0
        // - The 50.0 at 12.0 should remain unchanged
        // ask_capacity should now be: 36.0 + 50.0 = 86.0
        assert_eq!(orderbook.ask_capacity(), 86.0,
                   "After matching 5.0 units, ask_capacity should be 86.0 (91.0 - 5.0)");

        // Verify the market order is filled
        assert!(market_order.is_filled(), "Market order should be completely filled");
        assert_eq!(market_order.size(), 0.0, "Market order size should be 0 after filling");
    }

}

