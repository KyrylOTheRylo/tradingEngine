use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use super::orderbook::{BidOrAsk, FillReport, Order, OrderBook, Tick};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type MarketId = u32;

pub const PRICE_SCALE: i64 = 10_000;

pub fn price_to_tick(price: Decimal) -> Tick {
    let scaled = price * Decimal::from(PRICE_SCALE);
    let rounded = scaled.round_dp(0);
    rounded
        .to_i64()
        .unwrap_or_else(|| if scaled.is_sign_negative() { i64::MIN } else { i64::MAX })
}

pub fn tick_to_price(tick: Tick) -> Decimal {
    Decimal::from(tick) / Decimal::from(PRICE_SCALE)
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TradingPair {
    base: String,
    quote: String,
}
impl TradingPair {
    pub fn new(base: String, quote: String) -> TradingPair {
        TradingPair { base, quote }
    }
    pub fn to_string(&self) -> String {
        format!("{}_{}", self.base, self.quote)
    }
    pub fn get_pair(&self) -> Vec<String> {
        vec![self.base.clone(), self.quote.clone()]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    New,
    Open,
    PartiallyFilled,
    Filled,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderSnapshot {
    pub id: u64,
    pub user_id: String,
    pub pair: TradingPair,
    pub side: BidOrAsk,
    pub order_type: OrderType,
    pub price: Option<Tick>,
    pub original_size: f64,
    pub remaining_size: f64,
    pub filled_size: f64,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order: OrderSnapshot,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngineStats {
    pub fills_total: u64,
    pub resting_orders_consumed_total: u64,
    pub levels_crossed_total: u64,
    pub total_matched_qty: f64,
}

#[derive(Debug)]
pub struct MatchEngine {
    orderbooks: Vec<OrderBook>,
    markets: Vec<TradingPair>,
    market_index: HashMap<TradingPair, MarketId>,
    orders: HashMap<u64, OrderSnapshot>,
    next_order_id: u64,
    stats: EngineStats,
}

impl MatchEngine {
    pub fn new() -> MatchEngine {
        MatchEngine {
            orderbooks: Vec::new(),
            markets: Vec::new(),
            market_index: HashMap::new(),
            orders: HashMap::new(),
            next_order_id: 1,
            stats: EngineStats::default(),
        }
    }

    pub fn stats(&self) -> EngineStats {
        self.stats.clone()
    }

    pub fn reset_stats(&mut self) {
        self.stats = EngineStats::default();
    }

    pub fn next_order_id(&mut self) -> u64 {
        let id = self.next_order_id;
        self.next_order_id = self.next_order_id.saturating_add(1);
        id
    }

    fn ensure_order_identity(&mut self, order: &mut Order) {
        if order.id() == 0 {
            order.set_id(self.next_order_id());
        }
        if order.user_id().is_empty() {
            order.set_user_id("unknown".to_string());
        }
    }

    pub fn add_new_market(&mut self, pair: TradingPair) -> MarketId {
        if let Some(existing) = self.market_index.get(&pair) {
            return *existing;
        }
        let market_id = self.markets.len() as MarketId;
        self.markets.push(pair.clone());
        self.orderbooks.push(OrderBook::new());
        self.market_index.insert(pair, market_id);
        market_id
    }

    pub fn get_market_id(&self, pair: &TradingPair) -> Option<MarketId> {
        self.market_index.get(pair).copied()
    }

    pub fn trading_pair(&self, market_id: MarketId) -> Option<&TradingPair> {
        self.markets.get(market_id as usize)
    }

    fn snapshot_from_order(
        pair: TradingPair,
        order: &Order,
        order_type: OrderType,
        price: Option<Tick>,
        original_size: f64,
        status: OrderStatus,
    ) -> OrderSnapshot {
        let remaining_size = order.size();
        OrderSnapshot {
            id: order.id(),
            user_id: order.user_id().to_string(),
            pair,
            side: order.bid_or_ask(),
            order_type,
            price,
            original_size,
            remaining_size,
            filled_size: (original_size - remaining_size).max(0.0),
            status,
        }
    }

    fn apply_fill_snapshot(snapshot: &mut OrderSnapshot, filled_qty: f64) {
        let mut remaining = snapshot.remaining_size - filled_qty;
        if remaining < 0.0 {
            remaining = 0.0;
        }
        snapshot.remaining_size = remaining;
        snapshot.filled_size = (snapshot.original_size - remaining).max(0.0);
        snapshot.status = if remaining == 0.0 {
            OrderStatus::Filled
        } else {
            OrderStatus::PartiallyFilled
        };
    }

    fn market_message(side: BidOrAsk, report: &FillReport) -> String {
        if report.insufficient_liquidity {
            match side {
                BidOrAsk::Bid => "Not enough ask orders to fill this buy".to_string(),
                BidOrAsk::Ask => "Not enough bid orders to fill this sell".to_string(),
            }
        } else {
            let side_label = match side {
                BidOrAsk::Bid => "Bid",
                BidOrAsk::Ask => "Ask",
            };
            format!(
                "Successfully filled {} {} market orders",
                report.filled_qty, side_label
            )
        }
    }

    fn execute_market_order_by_id(
        &mut self,
        market_id: MarketId,
        order: &mut Order,
    ) -> Result<(OrderSnapshot, FillReport), String> {
        self.ensure_order_identity(order);
        let original_size = order.size();

        let pair = self
            .markets
            .get(market_id as usize)
            .cloned()
            .ok_or_else(|| format!("market id {} doesn't exist", market_id))?;

        let report = {
            let (orderbooks, orders) = (&mut self.orderbooks, &mut self.orders);
            let orderbook = orderbooks
                .get_mut(market_id as usize)
                .ok_or_else(|| format!("market id {} doesn't exist", market_id))?;

            let mut on_fill = |order_id: u64, filled_qty: f64| {
                if let Some(snapshot) = orders.get_mut(&order_id) {
                    Self::apply_fill_snapshot(snapshot, filled_qty);
                }
            };

            orderbook.fill_order_book_with_report(order, &mut on_fill)
        };

        self.stats.fills_total += report.fills_total;
        self.stats.resting_orders_consumed_total += report.resting_orders_consumed;
        self.stats.levels_crossed_total += report.levels_crossed;
        self.stats.total_matched_qty += report.total_matched_qty;

        let status = if report.filled_qty == 0.0 {
            OrderStatus::Rejected
        } else if report.fully_filled {
            OrderStatus::Filled
        } else {
            OrderStatus::PartiallyFilled
        };

        let snapshot = Self::snapshot_from_order(
            pair,
            order,
            OrderType::Market,
            None,
            original_size,
            status,
        );
        self.orders.insert(snapshot.id, snapshot.clone());

        Ok((snapshot, report))
    }

    pub fn fill_market_order_by_id(&mut self, market_id: MarketId, order: &mut Order) -> Result<String, String> {
        let response = self.fill_market_order_with_response_by_id(market_id, order)?;
        Ok(response.message)
    }

    pub fn fill_market_order_raw_by_id(&mut self, market_id: MarketId, order: &mut Order) -> Result<(), String> {
        let _ = self.execute_market_order_by_id(market_id, order)?;
        Ok(())
    }

    pub fn fill_market_order_with_response_by_id(
        &mut self,
        market_id: MarketId,
        order: &mut Order,
    ) -> Result<OrderResponse, String> {
        let (snapshot, report) = self.execute_market_order_by_id(market_id, order)?;
        let message = Self::market_message(snapshot.side, &report);

        Ok(OrderResponse {
            order: snapshot,
            message,
        })
    }

    pub fn fill_market_order(&mut self, pair: &TradingPair, order: &mut Order) -> Result<String, String> {
        let market_id = self
            .get_market_id(pair)
            .ok_or_else(|| format!("the orderbook {} doesn't exist ", pair.to_string()))?;
        self.fill_market_order_by_id(market_id, order)
    }

    pub fn fill_market_order_raw(&mut self, pair: &TradingPair, order: &mut Order) -> Result<(), String> {
        let market_id = self
            .get_market_id(pair)
            .ok_or_else(|| format!("the orderbook {} doesn't exist ", pair.to_string()))?;
        self.fill_market_order_raw_by_id(market_id, order)
    }

    pub fn fill_market_order_with_response(
        &mut self,
        pair: &TradingPair,
        order: &mut Order,
    ) -> Result<OrderResponse, String> {
        let market_id = self
            .get_market_id(pair)
            .ok_or_else(|| format!("the orderbook {} doesn't exist ", pair.to_string()))?;
        self.fill_market_order_with_response_by_id(market_id, order)
    }

    pub fn get_limits_for_a_pair(&self, pair: &TradingPair) -> Option<&OrderBook> {
        let market_id = self.get_market_id(pair)?;
        self.orderbooks.get(market_id as usize)
    }

    pub fn get_limits_for_market(&self, market_id: MarketId) -> Option<&OrderBook> {
        self.orderbooks.get(market_id as usize)
    }

    pub fn get_orderbooks(&self) -> Vec<Vec<String>> {
        self.markets
            .iter()
            .map(|pair: &TradingPair| pair.get_pair())
            .collect::<Vec<_>>()
    }

    pub fn get_order(&self, order_id: u64) -> Option<OrderSnapshot> {
        self.orders.get(&order_id).cloned()
    }

    pub fn get_orders_for_user(&self, user_id: &str) -> Vec<OrderSnapshot> {
        self.orders
            .values()
            .filter(|order| order.user_id == user_id)
            .cloned()
            .collect()
    }

    pub fn make_limit(&mut self, size: f64, bid_or_ask: BidOrAsk) -> Order {
        Order::new(size, bid_or_ask)
    }

    fn place_limit_order_internal_by_id(
        &mut self,
        market_id: MarketId,
        price_tick: Tick,
        mut order: Order,
    ) -> Result<OrderSnapshot, String> {
        self.ensure_order_identity(&mut order);

        let pair = self
            .markets
            .get(market_id as usize)
            .cloned()
            .ok_or_else(|| format!("market id {} doesn't exist", market_id))?;

        let snapshot = Self::snapshot_from_order(
            pair,
            &order,
            OrderType::Limit,
            Some(price_tick),
            order.size(),
            OrderStatus::Open,
        );

        {
            let orderbook = self
                .orderbooks
                .get_mut(market_id as usize)
                .ok_or_else(|| format!("market id {} doesn't exist", market_id))?;

            match order.bid_or_ask() {
                BidOrAsk::Ask => {
                    if let Some(best_bid) = orderbook.first_price_bid() {
                        if best_bid >= price_tick {
                            return Err(format!(
                                "You can not place a sell order on that price {:?}. Try a market order.",
                                tick_to_price(price_tick)
                            ));
                        }
                    }
                }
                BidOrAsk::Bid => {
                    if let Some(best_ask) = orderbook.first_price_ask() {
                        if best_ask <= price_tick {
                            return Err(format!(
                                "You can not place a buy order on that price {:?}. Try a market order.",
                                tick_to_price(price_tick)
                            ));
                        }
                    }
                }
            };

            orderbook.add_limit_order(price_tick, order);
        }

        self.orders.insert(snapshot.id, snapshot.clone());
        Ok(snapshot)
    }

    pub fn place_limit_order_by_id_tick(
        &mut self,
        market_id: MarketId,
        price_tick: Tick,
        order: Order,
    ) -> Result<String, String> {
        let response = self.place_limit_order_with_response_by_id_tick(market_id, price_tick, order)?;
        Ok(response.message)
    }

    pub fn place_limit_order_raw_by_id_tick(
        &mut self,
        market_id: MarketId,
        price_tick: Tick,
        order: Order,
    ) -> Result<(), String> {
        let _ = self.place_limit_order_internal_by_id(market_id, price_tick, order)?;
        Ok(())
    }

    pub fn place_limit_order_with_response_by_id_tick(
        &mut self,
        market_id: MarketId,
        price_tick: Tick,
        order: Order,
    ) -> Result<OrderResponse, String> {
        let snapshot = self.place_limit_order_internal_by_id(market_id, price_tick, order)?;
        let side_label = match snapshot.side {
            BidOrAsk::Ask => "Ask",
            BidOrAsk::Bid => "Bid",
        };
        let display_price = tick_to_price(price_tick);
        let message = format!(
            " received {} order with size {} in pair {} on price {}",
            side_label,
            snapshot.original_size,
            snapshot.pair.to_string(),
            display_price
        );

        Ok(OrderResponse {
            order: snapshot,
            message,
        })
    }

    pub fn place_limit_order_by_id(
        &mut self,
        market_id: MarketId,
        price: Decimal,
        order: Order,
    ) -> Result<String, String> {
        let price_tick = price_to_tick(price);
        self.place_limit_order_by_id_tick(market_id, price_tick, order)
    }

    pub fn place_limit_order_raw_by_id(
        &mut self,
        market_id: MarketId,
        price: Decimal,
        order: Order,
    ) -> Result<(), String> {
        let price_tick = price_to_tick(price);
        self.place_limit_order_raw_by_id_tick(market_id, price_tick, order)
    }

    pub fn place_limit_order_with_response_by_id(
        &mut self,
        market_id: MarketId,
        price: Decimal,
        order: Order,
    ) -> Result<OrderResponse, String> {
        let price_tick = price_to_tick(price);
        self.place_limit_order_with_response_by_id_tick(market_id, price_tick, order)
    }

    pub fn place_limit_order(&mut self, pair: &TradingPair, price: Decimal, order: Order) -> Result<String, String> {
        let market_id = self
            .get_market_id(pair)
            .ok_or_else(|| format!("the orderbook {} doesn't exist ", pair.to_string()))?;
        self.place_limit_order_by_id(market_id, price, order)
    }

    pub fn place_limit_order_raw(
        &mut self,
        pair: &TradingPair,
        price: Decimal,
        order: Order,
    ) -> Result<(), String> {
        let market_id = self
            .get_market_id(pair)
            .ok_or_else(|| format!("the orderbook {} doesn't exist ", pair.to_string()))?;
        self.place_limit_order_raw_by_id(market_id, price, order)
    }

    pub fn place_limit_order_with_response(
        &mut self,
        pair: &TradingPair,
        price: Decimal,
        order: Order,
    ) -> Result<OrderResponse, String> {
        let market_id = self
            .get_market_id(pair)
            .ok_or_else(|| format!("the orderbook {} doesn't exist ", pair.to_string()))?;
        self.place_limit_order_with_response_by_id(market_id, price, order)
    }
}
