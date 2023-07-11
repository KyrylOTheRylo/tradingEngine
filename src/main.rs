use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

mod engine;
use engine::{Price, Limit, Order, BidOrAsk};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {

    HttpResponse::Ok().body("SADAS")
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _a: Price = engine::Price::new(15.5);
    let mut lim: Limit = Limit::new(_a); 
    let ord: Order = Order::new(5.5, BidOrAsk::Ask);
    lim.add_order(ord);
    println!("{:?}", lim);
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