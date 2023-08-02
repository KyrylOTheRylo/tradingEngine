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

    pub fn fill_order_book(&mut self, market_order:&mut Order)  
    {
        match market_order.bid_or_ask 
        {
            BidOrAsk::Ask => {
                for limit_order in self.ask_limits() {
                    limit_order.fill_order( market_order);
                }
            }
            BidOrAsk::Bid => {
                for limit_order in self.bid_limits() {
                limit_order.fill_order( market_order);
            }}

        }
    }
    pub fn ask_limits(&mut self) -> Vec<&mut Limit> {
        self.asks.values_mut()
            .collect::<Vec<&mut Limit>>()
    }
    pub fn bid_limits(&mut self) -> Vec<&mut Limit> {
        self.bids.values_mut()
            .collect::<Vec<&mut Limit>>()
    }
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
    pub fn add_order(&mut self, price: f64, order: Order) {
        let price = Price::new(price);
        match order.bid_or_ask {
            BidOrAsk::Ask => {
                self.add_order_from_price_in_bids_or_asks(price, order, BidOrAsk::Ask);
            }
            BidOrAsk::Bid => {
                self.add_order_from_price_in_bids_or_asks(price, order, BidOrAsk::Bid);
            }
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
        }}
    pub fn is_filled(&self) -> bool {
        self.size == 0.0
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
    total_volume: f64,

}

impl Limit {        
    pub fn new(price: Price) -> Limit { 
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

    #[test]
    fn total_volume_test2() {
        let price: Price = Price::new(1000.0);
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
        let price: Price = Price::new(1000.0);
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
        let price: Price = Price::new(1000.0);
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
        let price = Price::new(1000.0);
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