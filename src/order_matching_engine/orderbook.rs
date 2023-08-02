#![allow(dead_code)]
use std::collections::HashMap;
use rust_decimal::prelude::*;

#[derive(Debug)]
pub struct OrderBook {
    asks: HashMap<Decimal, Limit>,
    bids: HashMap<Decimal, Limit>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            asks: HashMap::new(),
            bids: HashMap::new(),
        }}

    pub fn fill_order_book(&mut self, market_order:&mut Order)  {
        let limits: Vec<&mut Limit> = match  market_order.bid_or_ask {
            BidOrAsk::Ask => self.bid_limits(),
            BidOrAsk::Bid => self.ask_limits(),
            
        };

        for limit in limits 
        {
            limit.fill_order(market_order);
            if market_order.is_filled() {
                break;
            }
        }

        }

    pub fn ask_limits(&mut self) -> Vec<&mut Limit> {
        let mut limits: Vec<&mut Limit> = self.asks
        .values_mut()
        .collect::<Vec<&mut Limit>>();

        limits.sort_by(|a: &&mut Limit,
             b: &&mut Limit| a.price.cmp(&b.price));

        limits
    }
    pub fn bid_limits(&mut self) -> Vec<&mut Limit> {
        let mut limits: Vec<&mut Limit> = self.asks
        .values_mut()
        .collect::<Vec<&mut Limit>>();

        limits.sort_by(|a: &&mut Limit,
             b: &&mut Limit| b.price.cmp(&a.price));

        limits
    }
    fn add_order_from_price_in_bids_or_asks(&mut self, price: Decimal, order: Order, bid_or_ask: BidOrAsk)  {
        let limit_map: &mut HashMap<Decimal, Limit> = match bid_or_ask {
            BidOrAsk::Bid => &mut self.bids,
            BidOrAsk::Ask => &mut self.asks,
        };

        match limit_map.get_mut(&price) {
            Some(limit) => {
                limit.add_order(order)
            }
            None => {
                let mut lim: Limit = Limit::new(price);
                lim.add_order(order);
                limit_map.insert(price, lim);
            }
        }
    }
    pub fn add_order(&mut self, price: Decimal, order: Order) {
        
        match order.bid_or_ask {
            BidOrAsk::Ask => {
                self.add_order_from_price_in_bids_or_asks(price, order, BidOrAsk::Ask);
            }
            BidOrAsk::Bid => {
                self.add_order_from_price_in_bids_or_asks(price, order, BidOrAsk::Bid);
            }
        }
    
}}


#[derive(Debug, Hash, Clone, Copy)]
pub enum BidOrAsk {
    Bid,
    Ask  
}

#[derive(Debug, Clone, Copy)]
pub struct Order {
    size: f64,
    bid_or_ask: BidOrAsk, 
    
}

impl Order {
    pub fn new(size: f64, bid_or_ask: BidOrAsk) -> Order {
        Order {
            size: size,
            bid_or_ask: bid_or_ask
        }}
    pub fn is_filled(&self) -> bool {
        self.size == 0.0
    }}



#[derive(Debug)]
pub struct Limit {
    price: Decimal,
    orders: Vec<Order>,
    total_volume: f64,

}

impl Limit {        
    pub fn new(price: Decimal) -> Limit { 
        let total_volume: f64 = 0.0;
        Limit{
            
            price,
            orders: Vec::new(), 
            total_volume,
        }}
    
    
    pub fn total_volume(&self) -> f64 { self.total_volume}

    pub fn fill_order(&mut self, market_order: &mut Order) {
        let mut delete_order: Vec<usize> = Vec::new();

        for (n, limit_order) in self.orders.iter_mut().enumerate() {
            match market_order.size  >= limit_order.size {
                true => {
                    market_order.size -= &limit_order.size;
                    self.total_volume -= &limit_order.size;
                    limit_order.size = 0.0;
                    delete_order.push(n);


                },
                false => {
                    limit_order.size -= market_order.size;
                    self.total_volume -= market_order.size;
                    market_order.size = 0.0;
                }
            }

            if market_order.is_filled() {
                break;
            }

    }
    }
    pub fn add_order(&mut self, order: Order) {
        self.orders.push(order);
        self.total_volume += order.size;
        
    }

}

#[cfg(test)]
pub mod test {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn total_volume_test2() {
        let price: Decimal = dec!(1000.0);
        let mut limit: Limit = Limit::new(price);
        let buy_limit_order1 =
        Order::new( 50.0, BidOrAsk::Bid);
        let buy_limit_order2 =
        Order::new( 48.0, BidOrAsk::Bid);
        limit.add_order(buy_limit_order1);
        limit.add_order(buy_limit_order2);
        assert_eq!(limit.total_volume, 98.0);
       
    }

    #[test]
    fn total_volume_test() {
        let price: Decimal = dec!(1000.0);
        let mut limit: Limit = Limit::new(price);
        let buy_limit_order1 =
        Order::new( 50.0, BidOrAsk::Bid);
        let buy_limit_order2 =
        Order::new( 48.0, BidOrAsk::Bid);

        let mut  market_sell_order =
         Order::new( 51.0, BidOrAsk::Ask);

        limit.add_order(buy_limit_order1);
        limit.add_order(buy_limit_order2);
        limit.fill_order(&mut market_sell_order);
        assert_eq!(limit.total_volume, 98.0 - 51.0);
       
    }

    #[test]
    fn total_volume_test3() {
        let price: Decimal = dec!(1000.0);
        let mut limit: Limit = Limit::new(price);
        let buy_limit_order1 =
        Order::new( 50.0, BidOrAsk::Bid);
        let buy_limit_order2 =
        Order::new( 48.0, BidOrAsk::Bid);

        let mut  market_sell_order =
         Order::new( 151.0, BidOrAsk::Ask);

        limit.add_order(buy_limit_order1);
        limit.add_order(buy_limit_order2);
        limit.fill_order(&mut market_sell_order);
        assert_eq!(limit.total_volume, 0.0);
        assert_eq!(market_sell_order.size, 53.0 );
    }

    #[test]
    fn limit_order_fill() {
        let price: Decimal = dec!(1000.0);
        let mut limit = Limit::new(price) ;
    
        let buy_limit_order1 =
         Order::new( 50.0, BidOrAsk::Bid);
        let buy_limit_order2 =
         Order::new( 48.0, BidOrAsk::Bid);
        
        let mut  market_sell_order =
         Order::new( 51.0, BidOrAsk::Ask);
        limit.add_order(buy_limit_order1);
        limit.add_order(buy_limit_order2);


        limit.fill_order(&mut market_sell_order);
        println!("{:?}", limit);
    }
    
}