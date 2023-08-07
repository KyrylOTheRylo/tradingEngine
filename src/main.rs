use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::path::Path;
use std::sync::{Arc, Mutex};
mod order_matching_engine;
use order_matching_engine::orderbook::{Order, Limit,  OrderBook, BidOrAsk};
use order_matching_engine::engine::{TradingPair, MatchEngine};
use actix_web::{FromRequest, HttpRequest};


#[post("/create_limit_order/{base}_{quote}/{price}/{size}")]
async fn create_limit_order(data: web::Data<Arc<Mutex<MatchEngine>>>,
    params: web::Path<(String, String, String, String)>) -> impl Responder {
        //let pair: TradingPair = TradingPair::new(params.0 , params.1);
        //let order: Order  = Order::new();
        let engine: std::sync::MutexGuard<'_, MatchEngine> = data.lock().unwrap();
        //engine.place_limit_order(pair, price, order); 

        println!("{} {} {} {} ",params.0, params.1, params.2, params.3 );
        HttpResponse::Ok().body("SADAS")    
}

#[get("/get_list_of_pairs")]
async fn get_list_of_pairs(data: web::Data<Arc<Mutex<MatchEngine>>>) -> impl Responder {
    let answ: Vec<Vec<String>> = {
        let engine: std::sync::MutexGuard<'_, MatchEngine> = data.lock().unwrap();
        engine.get_orderbooks()
    };
    HttpResponse::Ok().json(answ.to_vec())
}

#[get("/get_limits_for_a_pair/{base}_{quote}")]
async fn get_limits_for_a_pair(data: web::Data<Arc<Mutex<MatchEngine>>>,
    params: web::Path<(String, String)>) -> impl Responder {
        let pair: TradingPair = TradingPair::new(params.0.to_string(), params.1.to_string());
    
    let engine: std::sync::MutexGuard<'_, MatchEngine> = data.lock().unwrap();
    let order_book = engine.get_limits_for_a_pair(pair);

    // Serialize OrderBook to JSON and then deserialize it back to create a deep copy
    match order_book {
        None => HttpResponse::Ok().json(""),
        _ => {
            let order_book_json = serde_json::to_string(&order_book).unwrap();
            let cloned_order_book: OrderBook = serde_json::from_str(&order_book_json).unwrap();

            HttpResponse::Ok().json(cloned_order_book)
        }
    }
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
    let mut engine: MatchEngine = MatchEngine::new();
    let btc_usd: TradingPair = TradingPair::new(String::from("btc"), String::from("usd"));
    let btc_eth: TradingPair = TradingPair::new(String::from("btc"), String::from("eth"));
    {
        engine.add_new_market(btc_usd.clone());
        engine.add_new_market(btc_eth.clone());
    }
    let data: web::Data<Arc<Mutex<MatchEngine>>> = web::Data::new(Arc::new(Mutex::new(engine)));
    let order: Order = Order::new(10.4, BidOrAsk::Ask);

    {
        let mut  engine: std::sync::MutexGuard<'_, MatchEngine> = data.lock().unwrap();
        engine.place_limit_order(btc_usd.clone(), dec!(10.3), order);
        let order2: Order = Order::new(10.5, BidOrAsk::Bid);
        let p: Result<String, String>  = engine.place_limit_order(btc_usd.clone(), dec!(10.5), order2);
        println!("{:?}", p.clone());
    }

    {
        let engine: std::sync::MutexGuard<'_, MatchEngine> = data.lock().unwrap();
        println!("{:?}",engine.get_limits_for_a_pair(btc_usd));
    }
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(create_limit_order)
            .service(get_list_of_pairs)
            .service(get_limits_for_a_pair)
            .service(echo)     
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}