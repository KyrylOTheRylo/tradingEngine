use rust_decimal::Decimal;

use super::orderbook::{OrderBook, Order, BidOrAsk};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
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
    pub fn get_pair(&self) -> Vec<String>{
        vec![self.base.clone(), self.quote.clone()]
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
 pub fn fill_market_order(&mut self,pair: TradingPair, order: &mut Order) -> Result<String, String> {
    match  self.orderbooks.get_mut(&pair) {
        Some(orderbook) => {
            return Ok(orderbook.fill_order_book(order));

        },
        None => {
            Err(format!("the orderbook {} doesn't exist ",  pair.to_string()))
        }
    }
 }
 pub fn get_limits_for_a_pair<'a>(&self, pair: TradingPair) -> Option<&OrderBook>{
    self.orderbooks.get(&pair)
 } 
 pub fn get_orderbooks(&self) -> Vec<Vec<String>> {
   self.orderbooks.keys().into_iter().map(|pair: &TradingPair| pair.get_pair()).collect::<Vec<_>>()}

 pub fn add_new_market(&mut self, pair: TradingPair) {

    self.orderbooks.insert(pair.clone(), OrderBook::new());
 }
 pub fn make_limit(&mut self, size: f64, bid_or_ask: BidOrAsk) -> Order {
    Order::new(size, bid_or_ask)
 }
 pub fn place_limit_order<'a>(&mut self, pair: TradingPair, price: Decimal, order: Order) -> Result<String, String> {
    match  self.orderbooks.get_mut(&pair) {
        Some(orderbook) => {
            match order.bid_or_ask() {
                BidOrAsk::Ask => {
                    if let Some(best_bid) = orderbook.first_price_bid() {
                        if best_bid >= price {
                            return Err(format!("You can not place a sell order on that price {:?}. Try a market order.", price));
                        }
                    }
                }
                BidOrAsk::Bid => {
                    if let Some(best_ask) = orderbook.first_price_ask() {
                        if best_ask <= price {
                            return Err(format!("You can not place a buy order on that price {:?}. Try a market order.", price));
                        }
                    }
                }
            };
            orderbook.add_limit_order(price, order);
            Ok(format!(" received {} order with size {} in pair {} on price {}", order.get_bid_or_ask(),
            order.size(),  pair.to_string(), price))
        }
        None => {
            Err(format!("the orderbook {} doesn't exist ",  pair.to_string()))
        }
        
    }
 }
 }