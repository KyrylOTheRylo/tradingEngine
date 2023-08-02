use rust_decimal::Decimal;

use super::orderbook::{OrderBook, Order};
use std::collections::HashMap;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct TradingPair {
    base: String,
    quote: String,
}
 impl TradingPair {
    pub fn new(base: String, quote: String) -> TradingPair{
        TradingPair{base, quote}
    }
    pub fn to_string(&self) -> String{
        format!("{}_{}", self.base, self.quote)
    }
 }
 #[derive(Debug)]
pub struct MatchEngine {
    orderbooks: HashMap<TradingPair, OrderBook>,
}

impl MatchEngine {
 pub fn new() -> MatchEngine{ 
    MatchEngine{orderbooks : HashMap::new()}   
}

 pub fn add_new_market(&mut self, pair: TradingPair) {

    self.orderbooks.insert(pair.clone(), OrderBook::new());
    println!("Opening new orderbook {:?}", pair.to_string())
 }
 pub fn place_limit_order(&mut self, pair: TradingPair, price: Decimal, order: Order) -> Result<(), String> {
    match  self.orderbooks.get_mut(&pair) {
        Some(orderbook) => {
            orderbook.add_limit_order(price, order);
            Ok(())
        }
        None => {
            Err(format!("the orderbook {} doesn't exist ",  pair.to_string()))
        }
        
    }
 }
 }