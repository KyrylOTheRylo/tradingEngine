#![allow(dead_code)]

use serde::{Serialize,Deserialize};
use std::collections::{BTreeMap, VecDeque};

pub type OrderId = u64;
pub type Tick = i64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RestingOrder {
    id: OrderId,
    qty: f64,
}

impl RestingOrder {
    pub fn new(id: OrderId, qty: f64) -> RestingOrder {
        RestingOrder { id, qty }
    }
    pub fn id(&self) -> OrderId { self.id }
    pub fn qty(&self) -> f64 { self.qty }
    pub fn set_qty(&mut self, qty: f64) { self.qty = qty; }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FillStats {
    pub fills_total: u64,
    pub resting_orders_consumed: u64,
    pub total_matched_qty: f64,
}

#[derive(Debug, Clone)]
pub struct FillReport {
    pub insufficient_liquidity: bool,
    pub fully_filled: bool,
    pub filled_qty: f64,
    pub remaining_qty: f64,
    pub fills_total: u64,
    pub resting_orders_consumed: u64,
    pub levels_crossed: u64,
    pub total_matched_qty: f64,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct OrderBook {
    asks: BTreeMap<Tick, Limit>,

    bids: BTreeMap<Tick, Limit>,
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

    pub fn first_price_ask(&self) -> Option<Tick>{
        self.asks.keys().next().copied()
    }
    pub fn first_price_bid(&self) -> Option<Tick>{
        self.bids.keys().next_back().copied()
    }

    pub fn fill_order_book(&mut self, market_order:&mut Order) -> String  {
        let report = self.fill_order_book_with_report(market_order, &mut |_order_id, _qty| {});
        if report.insufficient_liquidity {
            match market_order.bid_or_ask {
                BidOrAsk::Bid => String::from("Not enough ask orders to fill this buy"),
                BidOrAsk::Ask => String::from("Not enough bid orders to fill this sell"),
            }
        } else {
            let side = match market_order.bid_or_ask {
                BidOrAsk::Bid => "Bid",
                BidOrAsk::Ask => "Ask",
            };
            format!("Successfully filled {} {} market orders", report.filled_qty, side)
        }
    }

    pub fn fill_order_book_with_report<F>(&mut self, market_order: &mut Order, on_fill: &mut F) -> FillReport
    where
        F: FnMut(OrderId, f64),
    {
        let amount: f64 = market_order.size;

        match market_order.bid_or_ask {
            // Bid order (buy): match against asks (sellers) - need asks available
            BidOrAsk::Bid => {
                if self.ask_capacity < amount {
                    return FillReport {
                        insufficient_liquidity: true,
                        fully_filled: false,
                        filled_qty: 0.0,
                        remaining_qty: amount,
                        fills_total: 0,
                        resting_orders_consumed: 0,
                        levels_crossed: 0,
                        total_matched_qty: 0.0,
                    };
                }

                // BTreeMap::iter() naturally iterates from lowest to highest price (best asks first)
                let mut prices_to_remove = Vec::new();
                let mut fills_total = 0;
                let mut resting_orders_consumed = 0;
                let mut levels_crossed = 0;
                let mut total_matched_qty = 0.0;

                for (&price, limit) in self.asks.iter_mut() {
                    let stats = limit.fill_order(market_order, on_fill);
                    if stats.fills_total > 0 {
                        levels_crossed += 1;
                    }
                    fills_total += stats.fills_total;
                    resting_orders_consumed += stats.resting_orders_consumed;
                    total_matched_qty += stats.total_matched_qty;
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

                self.ask_capacity -= total_matched_qty;
                FillReport {
                    insufficient_liquidity: false,
                    fully_filled: market_order.is_filled(),
                    filled_qty: total_matched_qty,
                    remaining_qty: market_order.size,
                    fills_total,
                    resting_orders_consumed,
                    levels_crossed,
                    total_matched_qty,
                }
            },

            // Ask order (sell): match against bids (buyers) - need bids available
            BidOrAsk::Ask => {
                if self.bid_capacity < amount {
                    return FillReport {
                        insufficient_liquidity: true,
                        fully_filled: false,
                        filled_qty: 0.0,
                        remaining_qty: amount,
                        fills_total: 0,
                        resting_orders_consumed: 0,
                        levels_crossed: 0,
                        total_matched_qty: 0.0,
                    };
                }

                // Use reverse iterator: BTreeMap::iter().rev() goes from highest to lowest (best bids first)
                let mut prices_to_remove = Vec::new();
                let mut fills_total = 0;
                let mut resting_orders_consumed = 0;
                let mut levels_crossed = 0;
                let mut total_matched_qty = 0.0;

                for (&price, limit) in self.bids.iter_mut().rev() {
                    let stats = limit.fill_order(market_order, on_fill);
                    if stats.fills_total > 0 {
                        levels_crossed += 1;
                    }
                    fills_total += stats.fills_total;
                    resting_orders_consumed += stats.resting_orders_consumed;
                    total_matched_qty += stats.total_matched_qty;
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

                self.bid_capacity -= total_matched_qty;
                FillReport {
                    insufficient_liquidity: false,
                    fully_filled: market_order.is_filled(),
                    filled_qty: total_matched_qty,
                    remaining_qty: market_order.size,
                    fills_total,
                    resting_orders_consumed,
                    levels_crossed,
                    total_matched_qty,
                }
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
    fn add_order_from_price_in_bids_or_asks(&mut self, price: Tick, order: RestingOrder, bid_or_ask: BidOrAsk)  {
        let limit_map: &mut BTreeMap<Tick, Limit> = match bid_or_ask {
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
    pub fn add_limit_order(&mut self, price: Tick, order: Order) {
        
        match order.bid_or_ask {
            BidOrAsk::Ask => {
                let order_size = order.size();
                let resting_order = RestingOrder::new(order.id(), order_size);
                self.add_order_from_price_in_bids_or_asks(price, resting_order, BidOrAsk::Ask);
                self.ask_capacity += order_size;
            }
            BidOrAsk::Bid => {
                let order_size = order.size();
                let resting_order = RestingOrder::new(order.id(), order_size);
                self.add_order_from_price_in_bids_or_asks(price, resting_order, BidOrAsk::Bid);
                self.bid_capacity += order_size
            }
    }
}}


#[derive(Debug, Hash, Clone, Copy, Serialize,Deserialize)]
pub enum BidOrAsk {
    Bid,
    Ask  
}

#[derive(Debug, Clone, Serialize,Deserialize)]
pub struct Order {
    id: OrderId,
    user_id: String,
    size: f64,
    bid_or_ask: BidOrAsk, 
    
}

impl Order {
    pub fn new(size: f64, bid_or_ask: BidOrAsk) -> Order {
        Order {
            id: 0,
            user_id: String::new(),
            size: size,
            bid_or_ask: bid_or_ask
        }}
    pub fn new_with_meta(id: OrderId, user_id: String, size: f64, bid_or_ask: BidOrAsk) -> Order {
        Order {
            id,
            user_id,
            size,
            bid_or_ask,
        }
    }
    pub fn id(&self) -> OrderId { self.id }
    pub fn user_id(&self) -> &str { self.user_id.as_str() }
    pub fn set_id(&mut self, id: OrderId) { self.id = id; }
    pub fn set_user_id(&mut self, user_id: String) { self.user_id = user_id; }
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
    price: Tick,
    orders: VecDeque<RestingOrder>,
    total_volume: f64,

}

impl Limit {        
    pub fn new(price: Tick) -> Limit { 
        let total_volume: f64 = 0.0;
        Limit{
            
            price,
            orders: VecDeque::new(),
            total_volume,
        }}
    
    pub fn price(&self) -> Tick {
        self.price
    }

    pub fn total_volume(&self) -> f64 { self.total_volume}

    pub fn fill_order<F>(&mut self, market_order: &mut Order, on_fill: &mut F) -> FillStats
    where
        F: FnMut(OrderId, f64),
    {
        let mut stats = FillStats::default();
        while let Some(mut limit_order) = self.orders.pop_front() {
            if market_order.size >= limit_order.qty() {
                let filled_qty = limit_order.qty();
                market_order.size -= limit_order.qty();
                self.total_volume -= limit_order.qty();
                stats.fills_total += 1;
                stats.resting_orders_consumed += 1;
                stats.total_matched_qty += filled_qty;
                on_fill(limit_order.id(), filled_qty);
            } else {
                let filled_qty = market_order.size;
                limit_order.set_qty(limit_order.qty() - market_order.size);
                self.total_volume -= market_order.size;
                stats.fills_total += 1;
                stats.total_matched_qty += filled_qty;
                on_fill(limit_order.id(), filled_qty);
                market_order.size = 0.0;
                self.orders.push_front(limit_order);
            }

            if market_order.is_filled() {
                break;
            }
        }
        stats
    }
    pub fn add_order(&mut self, order: RestingOrder) {
        let order_size = order.qty();
        self.orders.push_back(order);
        self.total_volume += order_size;
        
    }

}
