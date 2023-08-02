use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

mod order_matching_engine;
use order_matching_engine::orderbook::{Order, Limit,  OrderBook, BidOrAsk};
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