use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

mod order_matching_engine;
use order_matching_engine::orderbook::{Order, Limit, Price, OrderBook, BidOrAsk};
use order_matching_engine::engine::{TradingPair, MatchEngine};




#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
#[post("/echo")]
async fn echo(_req_body: String) -> impl Responder {

    HttpResponse::Ok().body("SADAS")
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let sell_a: Price = Price::new(15.5);
    let lim: Limit = Limit::new(sell_a); 
    let sell_ord: Order = Order::new(5.5, BidOrAsk::Ask);
    let mut book: OrderBook = OrderBook::new();
    book.add_order(15.5, sell_ord ); 

    let buy_a: Price = Price::new(10.5);
    let lim: Limit = Limit::new(sell_a); 
    let buy_ord: Order = Order::new(5.5, BidOrAsk::Ask);
    book.add_order(10.5, buy_ord ); 

    let btc_usd: TradingPair = TradingPair::new(String::from("BTC"),String::from("USD"));


    let mut engine: MatchEngine = MatchEngine::new();
    engine.add_new_market(btc_usd);

    let pair_wrong: TradingPair = TradingPair::new(String::from("USD"),String::from("Uah"));
    println!("{:?}", engine.place_limit_order(pair_wrong, 10.5, buy_ord));






    println!("{:?}", book);
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}