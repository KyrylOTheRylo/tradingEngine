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

    pub fn bid_capacity(&self) -> f64 { return self.bid_capacity;}

    pub fn ask_capacity(&self) -> f64 { return  self.ask_capacity; }

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
        
    }
    pub fn size(&self) -> f64 {
        self.size}
    }

    



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

