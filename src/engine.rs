use std::collections::HashMap;

#[derive(Debug)]
pub struct OrderBook {
    asks: HashMap<Price, Limit>,
    bids: HashMap<Price, Limit>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            asks: HashMap::new(),
            bids: HashMap::new(),
        }}

    fn add_order_from_price_in_bids_or_asks(&mut self, price: Price, order: Order, bid_or_ask: BidOrAsk)  {
        let limit_map: &mut HashMap<Price, Limit> = match bid_or_ask {
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
    pub fn add_order(&mut self,price: f64, order: Order) {
        let price: Price= Price::new(price);
        match order.bid_or_ask {
            
            BidOrAsk::Ask => {
                self.add_order_from_price_in_bids_or_asks(price, order, order.bid_or_ask);
                
            },
            BidOrAsk::Bid => {
                self.add_order_from_price_in_bids_or_asks(price, order, order.bid_or_ask);
            },
            
    }
}}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct Price {
    mantica: u64,
    integral:  u64,
    fractional: u64, 

    
}
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
        }
}}

impl Price {

    pub fn new( price: f64) -> Price { 
        let mantica: u64= 1000000000u64;
        let integral: u64= price as u64;
        let fractional: u64 = ((price % 1.0) * mantica as f64) as u64;
        Price{
            mantica,
            integral,
            fractional,
        }
        
     }
}

#[derive(Debug)]
pub struct Limit {
    price: Price,
    orders: Vec<Order>,

}

impl Limit {        
    pub fn new(price: Price) -> Limit { 
        Limit{
            price,
            orders: Vec::new(), 
        }}
    pub fn add_order(&mut self, order: Order) {
        self.orders.push(order);
        
    }

}