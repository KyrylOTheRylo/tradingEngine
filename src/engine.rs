#[derive(Debug, Hash)]
pub struct Price {
    mantica: u64,
    integral:  u64,
    fractional: u64, 

    
}
#[derive(Debug, Hash)]
pub enum BidOrAsk {
    Bid,
    Ask  
}

#[derive(Debug)]
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