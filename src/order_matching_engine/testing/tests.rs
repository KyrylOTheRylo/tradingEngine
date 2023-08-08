


#[cfg(test)]
pub mod test {
    use crate::order_matching_engine::orderbook::{Order, Limit,  OrderBook, BidOrAsk};
    use crate::order_matching_engine::engine::{TradingPair, MatchEngine};
    
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn order_book_test() {
        let mut orderbook: OrderBook = OrderBook::new();
        orderbook.add_limit_order(dec!(500.0), Order::new(10.0, BidOrAsk::Ask)) ;
        orderbook.add_limit_order(dec!(400.0), Order::new(10.0, BidOrAsk::Ask)) ;
        orderbook.add_limit_order(dec!(200.0), Order::new(10.0, BidOrAsk::Ask)) ;
        orderbook.add_limit_order(dec!(300.0), Order::new(10.0, BidOrAsk::Ask)) ;

        let mut market_order: Order = Order::new(12.1, BidOrAsk::Bid);
        orderbook.fill_order_book(&mut market_order);
        orderbook.add_limit_order(dec!(200.0), Order::new(10.0, BidOrAsk::Bid)) ;

        orderbook.add_limit_order(dec!(10.0), Order::new(10.0, BidOrAsk::Bid)) ;
        let mut market_order2: Order = Order::new(12.1, BidOrAsk::Ask);
        orderbook.fill_order_book(&mut market_order2);
        println!("{:?}", orderbook);
        
        assert_eq!(orderbook.bid_capacity(), 27.9);
        assert_eq!(orderbook.ask_capacity(), 7.9);
    
    }


    #[test]
    fn total_volume_test2() {
        let price: Decimal = dec!(1000.0);
        let mut limit: Limit = Limit::new(price);
        let buy_limit_order1: Order =
        Order::new( 50.0, BidOrAsk::Bid);
        let buy_limit_order2: Order =
        Order::new( 48.0, BidOrAsk::Bid);
        limit.add_order(buy_limit_order1);
        limit.add_order(buy_limit_order2);
        assert_eq!(limit.total_volume(), 98.0);
       
    }

    #[test]
    fn total_volume_test() {
        let price: Decimal = dec!(1000.0);
        let mut limit: Limit = Limit::new(price);
        let buy_limit_order1: Order =
        Order::new( 50.0, BidOrAsk::Bid);
        let buy_limit_order2 =
        Order::new( 48.0, BidOrAsk::Bid);

        let mut  market_sell_order: Order =
         Order::new( 51.0, BidOrAsk::Ask);

        limit.add_order(buy_limit_order1);
        limit.add_order(buy_limit_order2);
        limit.fill_order(&mut market_sell_order);
        assert_eq!(limit.total_volume(), 98.0 - 51.0);
       
    }

    #[test]
    fn total_volume_test3() {
        let price: Decimal = dec!(1000.0);
        let mut limit: Limit = Limit::new(price);
        let buy_limit_order1: Order =
        Order::new( 50.0, BidOrAsk::Bid);
        let buy_limit_order2: Order =
        Order::new( 48.0, BidOrAsk::Bid);

        let mut  market_sell_order: Order =
         Order::new( 90.0, BidOrAsk::Ask);

        limit.add_order(buy_limit_order1);
        limit.add_order(buy_limit_order2);
        limit.fill_order(&mut market_sell_order);
        println!("{:?}", market_sell_order);
    }

    #[test]
    fn limit_order_fill() {
        let price: Decimal = dec!(1000.0);
        let mut limit: Limit = Limit::new(price) ;
    
        let buy_limit_order1: Order =
         Order::new( 50.0, BidOrAsk::Bid);
        let buy_limit_order2: Order =
         Order::new( 48.0, BidOrAsk::Bid);
        
        let mut  market_sell_order: Order =
         Order::new( 51.0, BidOrAsk::Ask);
        limit.add_order(buy_limit_order1);
        limit.add_order(buy_limit_order2);


        limit.fill_order(&mut market_sell_order);
        println!("{:?}", limit);
    }

    #[test]
    fn engine_all_orderbooks_test() {
        let mut engine: MatchEngine  = MatchEngine::new();

        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        let btc_eth = TradingPair::new(String::from("btc"), String::from("eth"));
        engine.add_new_market(btc_usd.clone());
        let order: Order = Order::new(10.4, BidOrAsk::Ask);
        engine.place_limit_order(btc_usd.clone(), dec!(10.3), order);
        
        let order2: Order = Order::new(10.5, BidOrAsk::Bid);
        engine.place_limit_order(btc_usd.clone(), dec!(10.5), order2);
        engine.get_orderbooks();
        println!("{:?}",engine.get_limits_for_a_pair(btc_usd));

        assert_eq!(engine.get_orderbooks().len(), 1);

    
    }

    #[test]
    fn market_order_test() {
        let mut engine: MatchEngine  = MatchEngine::new();

        let btc_usd = TradingPair::new(String::from("btc"), String::from("usd"));
        let btc_eth = TradingPair::new(String::from("btc"), String::from("eth"));
        engine.add_new_market(btc_usd.clone());
        let order: Order = Order::new(10.4, BidOrAsk::Ask);
        engine.place_limit_order(btc_usd.clone(), dec!(10.3), order);
        
        let order2: Order = Order::new(10.5, BidOrAsk::Bid);
        println!("{:?}",engine.place_limit_order(btc_usd.clone(), dec!(10.2), order2));

        let mut market_order = Order::new(10.5, BidOrAsk::Ask);
        let mut market_order2 = Order::new(10.4, BidOrAsk::Bid);
        engine.fill_market_order(btc_usd.clone() ,&mut market_order);
        engine.fill_market_order(btc_usd.clone() ,&mut market_order2);

        assert_eq!(engine.get_limits_for_a_pair(btc_usd).unwrap().bid_capacity(), 0.0);  

    
    }

}