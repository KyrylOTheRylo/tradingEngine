use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use rust_decimal_macros::dec;
use rust_decimal::Decimal;
use std::sync::{Arc, Mutex};
mod order_matching_engine;
use order_matching_engine::orderbook::{Order,  OrderBook, BidOrAsk};
use order_matching_engine::engine::{TradingPair, MatchEngine};



#[post("/create_limit_order/{base}_{quote}/{buy_or_sell}/{price}/{size}")]
async fn create_limit_order(data: web::Data<Arc<Mutex<MatchEngine>>>,
    params: web::Path<(String, String, String, String, String)>) -> impl Responder {
        
        //let order: Order  = Order::new();
        let price_or_wrong: String = params.3.to_string();
        
        match price_or_wrong.parse::<f64>() {
            Ok(price) => {
                let size_or_wrong = params.4.to_string();
                match size_or_wrong.parse::<f64>() {
                    Ok(size) => {
                        match params.2.as_str(){
                            "buy" => {
                                let pair: TradingPair = TradingPair::new(params.0.to_string() , params.1.to_string());
                                let order: Order  = Order::new(size, BidOrAsk::Bid);
                                let mut engine: std::sync::MutexGuard<'_, MatchEngine> = data.lock().unwrap();
                                let tmp_price: Decimal = Decimal::from_f64_retain(price).unwrap();
                                
                                
                                match engine.place_limit_order(pair, tmp_price, order){
                                    Ok(answ) => {return HttpResponse::Ok().body(answ);}
                                    Err(error_msg) => {return HttpResponse::Ok().body(error_msg);}
                                } 
                            },
                            "sell" => {
                                let pair: TradingPair = TradingPair::new(params.0.to_string() , params.1.to_string());
                                let order: Order  = Order::new(size, BidOrAsk::Ask);
                                let mut engine: std::sync::MutexGuard<'_, MatchEngine> = data.lock().unwrap();
                                let tmp_price: Decimal = Decimal::from_f64_retain(price).unwrap();
                                
                                
                                match engine.place_limit_order(pair, tmp_price, order){
                                    Ok(answ) => {return HttpResponse::Ok().body(answ);}
                                    Err(error_msg) => {return HttpResponse::Ok().body(error_msg);}
                                } 
                            },
                            _ => return HttpResponse::Ok().body("Wrong order type (should be buy or sell)"),
                        }
                    }
                    Err(_) => {return HttpResponse::Ok().body("Wrong size format")  ;}
                    }
                
            }
            Err(_) => {
                // Parsing failed, return an HTTP response with an error message.
                return HttpResponse::Ok().body("Invalid price format");
            }
        }
            

       
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
    let order_book: Option<&OrderBook> = engine.get_limits_for_a_pair(pair);

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