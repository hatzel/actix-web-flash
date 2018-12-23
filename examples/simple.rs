use actix_web::{http, server, App, Responder, HttpResponse, HttpRequest};
use actix_web_flash::{FlashMessage, FlashResponse};

fn show_flash(flash: FlashMessage<String>) -> impl Responder {
    flash.into_inner()
}

fn set_flash(_req: &HttpRequest) -> FlashResponse<HttpResponse, String> {
    FlashResponse::flash(
        HttpResponse::SeeOther()
            .header(http::header::LOCATION, "/show_flash")
            .finish(),
        FlashMessage::new("This is the message".to_owned())
    )
}

fn main() {
    server::new(
        || App::new()
            .route("/show_flash", http::Method::GET, show_flash)
            .resource("/set_flash", |r| r.f(set_flash)))
        .bind("127.0.0.1:8080").unwrap()
        .run();
}
