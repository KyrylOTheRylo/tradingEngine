#![allow(dead_code)]

use rust_decimal::prelude::*;
use serde::{Serialize,Deserialize};
use std::collections::BTreeMap;

#[derive(Debug,Serialize,Deserialize)]
pub struct OrderBook {
    asks: BTreeMap<Decimal, Limit>,

    bids: BTreeMap<Decimal, Limit>,
    ask_capacity: f64,
    bid_capacity: f64,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            asks: BTreeMap::new(),

            bids: BTreeMap::new(),
            ask_capacity : 0.0,
            bid_capacity : 0.0,
        }}
    
    pub fn bid_capacity(&self) -> f64 { return self.bid_capacity;}

    pub fn ask_capacity(&self) -> f64 { return  self.ask_capacity; }

    pub fn first_price_ask(&self) -> Option<Decimal>{
        self.asks.keys().next().copied()
    }
    pub fn first_price_bid(&self) -> Option<Decimal>{
        self.bids.keys().next_back().copied()
    }

    pub fn fill_order_book(&mut self, market_order:&mut Order) -> String  {
        let amount: f64 = market_order.size;

        match market_order.bid_or_ask {
            // Bid order (buy): match against asks (sellers) - need asks available
            BidOrAsk::Bid => {
                if self.ask_capacity < amount {
                    return String::from("Not enough ask orders to fill this buy");
                }

                // BTreeMap::iter() naturally iterates from lowest to highest price (best asks first)
                let mut prices_to_remove = Vec::new();

                for (&price, limit) in self.asks.iter_mut() {
                    limit.fill_order(market_order);
                    if limit.total_volume() == 0.0 {
                        prices_to_remove.push(price);
                    }
                    if market_order.is_filled() {
                        break;
                    }
                }

                // Remove empty price levels
                for price in prices_to_remove {
                    self.asks.remove(&price);
                }

                self.ask_capacity -= amount;
                format!("Successfully filled {} Bid market orders", amount)
            },

            // Ask order (sell): match against bids (buyers) - need bids available
            BidOrAsk::Ask => {
                if self.bid_capacity < amount {
                    return String::from("Not enough bid orders to fill this sell");
                }

                // Use reverse iterator: BTreeMap::iter().rev() goes from highest to lowest (best bids first)
                let mut prices_to_remove = Vec::new();

                for (&price, limit) in self.bids.iter_mut().rev() {
                    limit.fill_order(market_order);
                    if limit.total_volume() == 0.0 {
                        prices_to_remove.push(price);
                    }
                    if market_order.is_filled() {
                        break;
                    }
                }

                // Remove empty price levels
                for price in prices_to_remove {
                    self.bids.remove(&price);
                }

                self.bid_capacity -= amount;
                format!("Successfully filled {} Ask market orders", amount)
            },
        }
    }

    pub fn ask_limits(&self) -> Vec<&Limit> {
        self.asks.values().collect()
    }
        
    pub fn bid_limits(&self) -> Vec<&Limit> {
        // Return bids in descending order (highest first)
        self.bids.values().rev().collect()
    }
    fn add_order_from_price_in_bids_or_asks(&mut self, price: Decimal, order: Order, bid_or_ask: BidOrAsk)  {
        let limit_map: &mut BTreeMap<Decimal, Limit> = match bid_or_ask {
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
                self.ask_capacity += order.size;
            }
            BidOrAsk::Bid => {
                self.add_order_from_price_in_bids_or_asks(price, order, BidOrAsk::Bid);
                self.bid_capacity += order.size
            }
    }
}}


#[derive(Debug, Hash, Clone, Copy, Serialize,Deserialize)]
pub enum BidOrAsk {
    Bid,
    Ask  
}

#[derive(Debug, Clone, Copy,Serialize,Deserialize)]
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
    pub fn bid_or_ask(&self) -> BidOrAsk {self.bid_or_ask} 
    pub fn get_bid_or_ask(&self) -> String {  match self.bid_or_ask {
        BidOrAsk::Ask => return "Ask".to_string(),
        BidOrAsk::Bid => return "Bid".to_string()
    };
    }
    }
    

    



#[derive(Debug,Serialize,Deserialize)]
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
    
    pub fn price(&self) -> Decimal {
        self.price
    }

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
        for &index in delete_order.iter() {
            self.orders.remove(index);
        }
    }
    pub fn add_order(&mut self, order: Order) {
        self.orders.push(order);
        self.total_volume += order.size;
        
    }

}

