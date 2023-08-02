#![allow(dead_code)]
use std::collections::HashMap;
use rust_decimal::prelude::*;

#[derive(Debug)]
pub struct OrderBook {
    asks: HashMap<Decimal, Limit>,
    
    bids: HashMap<Decimal, Limit>,
    ask_capacity: f64,
    bid_capacity: f64,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            asks: HashMap::new(),

            bids: HashMap::new(),
            ask_capacity : 0.0,
            bid_capacity : 0.0,
        }}

    pub fn fill_order_book(&mut self, market_order:&mut Order) -> String  {
        let amount: f64 =market_order.size; 

        let limits: Vec<&mut Limit> = match  market_order.bid_or_ask {
            BidOrAsk::Ask => 
            if self.ask_capacity < amount {return String::from("Not enough bid orders")}
            else { self.bid_limits()},
            
            BidOrAsk::Bid => 
            if self.bid_capacity < amount {return String::from("Not enough ask orders")}
            else {self.ask_limits()},
            
        };
        let mut answ = String::new();
        for limit in limits 
        {
            limit.fill_order(market_order);
            
              
            if market_order.is_filled() {
                match market_order.bid_or_ask { 
                    BidOrAsk::Ask => {
                        self.ask_capacity -= amount;
                        answ = format!("Succesfully filled {} Ask market orders  ", amount);
                    },
                    BidOrAsk::Bid => {
                        self.bid_capacity -= amount;
                        answ = format!("Succesfully filled {} Bid market orders  ", amount);
                    },
                }
                break;
            }
        }
        return answ;

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
        let mut limits: Vec<&mut Limit> = self.bids
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
                limit.add_order(order);
               
            }
            None => {
                let mut lim: Limit = Limit::new(price);
                lim.add_order(order);
                limit_map.insert(price, lim);
                
            }
        }
    }
    pub fn add_limit_order(&mut self, price: Decimal, order: Order) {
        
        match order.bid_or_ask {
            BidOrAsk::Ask => {
                self.add_order_from_price_in_bids_or_asks(price, order, BidOrAsk::Ask);
                self.bid_capacity += order.size;
            }
            BidOrAsk::Bid => {
                self.add_order_from_price_in_bids_or_asks(price, order, BidOrAsk::Bid);
                self.ask_capacity += order.size
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
    fn order_book_test() {
        let mut orderbook = OrderBook::new();
        orderbook.add_limit_order(dec!(500.0), Order::new(10.0, BidOrAsk::Ask)) ;
        orderbook.add_limit_order(dec!(100.0), Order::new(10.0, BidOrAsk::Ask)) ;
        orderbook.add_limit_order(dec!(200.0), Order::new(10.0, BidOrAsk::Ask)) ;
        orderbook.add_limit_order(dec!(300.0), Order::new(10.0, BidOrAsk::Ask)) ;

        let mut market_order = Order::new(12.1, BidOrAsk::Bid);
        orderbook.fill_order_book(&mut market_order);
        orderbook.add_limit_order(dec!(100.0), Order::new(10.0, BidOrAsk::Bid)) ;

        orderbook.add_limit_order(dec!(10.0), Order::new(10.0, BidOrAsk::Bid)) ;
        let mut market_order2 = Order::new(12.1, BidOrAsk::Ask);
        orderbook.fill_order_book(&mut market_order2);

        
        println!("{:?}", orderbook.bid_limits());
        //assert_eq!(orderbook.bid_limits().get(0).unwrap().orders, BidOrAsk::Bid);
    }


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