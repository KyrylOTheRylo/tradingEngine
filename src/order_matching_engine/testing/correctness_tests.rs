// Comprehensive correctness tests for the order matching engine
// Tests the scenario that was previously failing

#[cfg(test)]
mod correctness_tests {
    use crate::order_matching_engine::orderbook::{Order, BidOrAsk};
    use crate::order_matching_engine::engine::{TradingPair, MatchEngine, price_to_tick};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn test_market_order_matching_scenario() {
        // Test the exact scenario from the bug report:
        // ASKS: price = 11.3 with amount 40.0
        //       price = 12.0 with amount 50.0
        //       price = 11.0 with amount 1.0
        // BIDS: price = 10.0 with amount 100.0
        //
        // Market BUY 5 should:
        // - Fill 1.0 at 11.0
        // - Fill 4.0 at 11.3
        // - NOT fill at 12.0

        let mut engine = MatchEngine::new();
        let btc_usd = TradingPair::new("btc".to_string(), "usd".to_string());
        engine.add_new_market(btc_usd.clone());

        // Add asks in non-sorted order
        let ask_11_3 = Order::new(40.0, BidOrAsk::Ask);
        let _ = engine.place_limit_order(&btc_usd, dec!(11.3), ask_11_3);

        let ask_12_0 = Order::new(50.0, BidOrAsk::Ask);
        let _ = engine.place_limit_order(&btc_usd, dec!(12.0), ask_12_0);

        let ask_11_0 = Order::new(1.0, BidOrAsk::Ask);
        let _ = engine.place_limit_order(&btc_usd, dec!(11.0), ask_11_0);

        // Add bid
        let bid_10_0 = Order::new(100.0, BidOrAsk::Bid);
        let _ = engine.place_limit_order(&btc_usd, dec!(10.0), bid_10_0);

        // Execute market buy for 5
        let mut market_buy = Order::new(5.0, BidOrAsk::Bid);
        let _ = engine.fill_market_order(&btc_usd, &mut market_buy);

        // Verify the market order was fully filled
        assert!(market_buy.is_filled(), "Market order should be fully filled");
        assert_eq!(market_buy.size(), 0.0, "Market order size should be 0 after filling");

        // Get the orderbook to check remaining orders
        let orderbook = engine.get_limits_for_a_pair(&btc_usd).unwrap();

        // Check asks: should have 11.0 gone (filled), 11.3 with 36.0 left, 12.0 with 50.0
        let ask_limits = orderbook.ask_limits();

        // Should have 2 ask levels remaining (11.3 and 12.0)
        assert_eq!(ask_limits.len(), 2, "Should have 2 ask levels remaining");

        // Find and verify each level
        let mut ask_11_3_found = false;
        let mut ask_12_0_found = false;

        for limit in ask_limits {
            if limit.price() == price_to_tick(dec!(11.3)) {
                ask_11_3_found = true;
                assert_eq!(limit.total_volume(), 36.0, "11.3 ask should have 36.0 remaining (40 - 4)");
            }
            if limit.price() == price_to_tick(dec!(12.0)) {
                ask_12_0_found = true;
                assert_eq!(limit.total_volume(), 50.0, "12.0 ask should have 50.0 remaining");
            }
        }

        assert!(ask_11_3_found, "Should find 11.3 ask level");
        assert!(ask_12_0_found, "Should find 12.0 ask level");

        // Check bids: should have bid 10.0 still at 100.0
        let bid_limits = orderbook.bid_limits();
        assert_eq!(bid_limits.len(), 1, "Should have 1 bid level");
        assert_eq!(bid_limits[0].price(), price_to_tick(dec!(10.0)), "Bid should be at 10.0");
        assert_eq!(bid_limits[0].total_volume(), 100.0, "Bid should still have 100.0");
    }

    #[test]
    fn test_asks_sorted_ascending() {
        // Verify that asks are always in ascending order (lowest first)
        let mut engine = MatchEngine::new();
        let btc_usd = TradingPair::new("btc".to_string(), "usd".to_string());
        engine.add_new_market(btc_usd.clone());

        // Add asks in random order
        let prices = vec![dec!(105.0), dec!(100.0), dec!(102.0), dec!(101.0), dec!(104.0)];

        for price in prices {
            let order = Order::new(10.0, BidOrAsk::Ask);
            let _ = engine.place_limit_order(&btc_usd, price, order);
        }

        // Get orderbook and verify ask order
        let orderbook = engine.get_limits_for_a_pair(&btc_usd).unwrap();
        let asks = orderbook.ask_limits();

        // Verify asks are sorted low to high
        let mut last_price = i64::MIN;
        for ask in asks {
            assert!(ask.price() >= last_price, "Asks should be sorted in ascending order");
            last_price = ask.price();
        }
    }

    #[test]
    fn test_bids_sorted_descending() {
        // Verify that bids are always in descending order (highest first)
        let mut engine = MatchEngine::new();
        let btc_usd = TradingPair::new("btc".to_string(), "usd".to_string());
        engine.add_new_market(btc_usd.clone());

        // Add bids in random order
        let prices = vec![dec!(95.0), dec!(100.0), dec!(98.0), dec!(99.0), dec!(96.0)];

        for price in prices {
            let order = Order::new(10.0, BidOrAsk::Bid);
            let _ = engine.place_limit_order(&btc_usd, price, order);
        }

        // Get orderbook and verify bid order
        let orderbook = engine.get_limits_for_a_pair(&btc_usd).unwrap();
        let bids = orderbook.bid_limits();

        // Verify bids are sorted high to low
        let mut last_price = i64::MAX;
        for bid in bids {
            assert!(bid.price() <= last_price, "Bids should be sorted in descending order");
            last_price = bid.price();
        }
    }

    #[test]
    fn test_partial_fill_at_multiple_levels() {
        // Market order that fills across multiple price levels
        let mut engine = MatchEngine::new();
        let btc_usd = TradingPair::new("btc".to_string(), "usd".to_string());
        engine.add_new_market(btc_usd.clone());

        // Add asks: 100@10, 100@11, 100@12, 100@13
        for price in 10..=13 {
            let order = Order::new(100.0, BidOrAsk::Ask);
            let _ = engine.place_limit_order(&btc_usd, Decimal::from(price), order);
        }

        // Market buy 250 - should fill:
        // 100 @ 10
        // 100 @ 11
        // 50 @ 12
        let mut market_buy = Order::new(250.0, BidOrAsk::Bid);
        let _ = engine.fill_market_order(&btc_usd, &mut market_buy);

        assert!(market_buy.is_filled(), "Market order should be fully filled");

        let orderbook = engine.get_limits_for_a_pair(&btc_usd).unwrap();
        let asks = orderbook.ask_limits();

        // Should have asks at 12 (50 left) and 13 (100 left)
        assert_eq!(asks.len(), 2, "Should have 2 ask levels remaining");

        let mut ask_12_found = false;
        let mut ask_13_found = false;

        for ask in asks {
            if ask.price() == price_to_tick(Decimal::from(12)) {
                ask_12_found = true;
                assert_eq!(ask.total_volume(), 50.0, "Ask at 12 should have 50.0 left");
            }
            if ask.price() == price_to_tick(Decimal::from(13)) {
                ask_13_found = true;
                assert_eq!(ask.total_volume(), 100.0, "Ask at 13 should have 100.0 left");
            }
        }

        assert!(ask_12_found && ask_13_found, "Should have correct remaining asks");
    }

    #[test]
    fn test_insufficient_liquidity() {
        // Market order with insufficient liquidity should fail
        let mut engine = MatchEngine::new();
        let btc_usd = TradingPair::new("btc".to_string(), "usd".to_string());
        engine.add_new_market(btc_usd.clone());

        // Add only 100 worth of asks
        let order = Order::new(100.0, BidOrAsk::Ask);
        let _ = engine.place_limit_order(&btc_usd, dec!(10.0), order);

        // Try to buy 200
        let mut market_buy = Order::new(200.0, BidOrAsk::Bid);
        let result = engine.fill_market_order(&btc_usd, &mut market_buy);

        match result {
            Ok(msg) => {
                // Should still return an error message about insufficient liquidity
                assert!(msg.contains("Not enough"), "Should mention insufficient liquidity");
            }
            Err(msg) => {
                assert!(msg.contains("Not enough"), "Should mention insufficient liquidity");
            }
        }
    }
}

