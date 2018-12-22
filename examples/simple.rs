use actix_web::{http, server, App, Path, Responder};
use actix_web_flash::Flash;

fn index(flash: Flash<String>) -> impl Responder {
    println!("flash: {:?}", flash);
    "".to_owned()
}

fn main() {
    server::new(
        || App::new()
            .route("/flash", http::Method::GET, index))
        .bind("127.0.0.1:8080").unwrap()
        .run();
}
