use crate::order_matching_engine::{orderbook::OrderBook,orderbook::Order, orderbook::BidOrAsk, orderbook::Limit};
use rust_decimal::prelude::*;
#[cfg(test)]
pub mod test {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn order_book_test() {
        let mut orderbook = OrderBook::new();
        orderbook.add_limit_order(dec!(500.0), Order::new(10.0, BidOrAsk::Ask)) ;
        orderbook.add_limit_order(dec!(100.0), Order::new(10.0, BidOrAsk::Ask)) ;
        orderbook.add_limit_order(dec!(200.0), Order::new(10.0, BidOrAsk::Ask)) ;
        orderbook.add_limit_order(dec!(300.0), Order::new(10.0, BidOrAsk::Ask)) ;

        let mut market_order: Order = Order::new(12.1, BidOrAsk::Bid);
        orderbook.fill_order_book(&mut market_order);
        orderbook.add_limit_order(dec!(100.0), Order::new(10.0, BidOrAsk::Bid)) ;

        orderbook.add_limit_order(dec!(10.0), Order::new(10.0, BidOrAsk::Bid)) ;
        let mut market_order2: Order = Order::new(12.1, BidOrAsk::Ask);
        orderbook.fill_order_book(&mut market_order2);

        
        assert_eq!(orderbook.bid_capacity(), 27.9);
        assert_eq!(orderbook.ask_capacity(), 7.9);
    
    }


    #[test]
    fn total_volume_test2() {
        let price: Decimal = dec!(1000.0);
        let mut limit: Limit = Limit::new(price);
        let buy_limit_order1: Order =
        Order::new( 50.0, BidOrAsk::Bid);
        let buy_limit_order2 =
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
         Order::new( 151.0, BidOrAsk::Ask);

        limit.add_order(buy_limit_order1);
        limit.add_order(buy_limit_order2);
        limit.fill_order(&mut market_sell_order);
        assert_eq!(limit.total_volume(), 0.0);
        assert_eq!(market_sell_order.size(), 53.0 );
    }

    #[test]
    fn limit_order_fill() {
        let price: Decimal = dec!(1000.0);
        let mut limit = Limit::new(price) ;
    
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
    
}