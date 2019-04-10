use actix_web::{http, server, App, HttpRequest, HttpResponse, Responder};
use actix_web_flash::{FlashMessage, FlashResponse, FlashMiddleware};

fn show_flash(flash: FlashMessage<String>) -> impl Responder {
    flash.into_inner()
}

fn set_flash(_req: &HttpRequest) -> FlashResponse<HttpResponse, String> {
    FlashResponse::with_redirect("This is the message".to_owned(), "/show_flash")
}

fn main() {
    server::new(|| {
        App::new()
            .middleware(FlashMiddleware::default())
            .route("/show_flash", http::Method::GET, show_flash)
            .resource("/set_flash", |r| r.f(set_flash))
    }).bind("127.0.0.1:8080")
        .unwrap()
        .run();
}
