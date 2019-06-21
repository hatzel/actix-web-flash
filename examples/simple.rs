use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash::{FlashMessage, FlashMiddleware, FlashResponse};

fn show_flash(flash: FlashMessage<String>) -> impl Responder {
    flash.into_inner()
}

fn set_flash(_req: HttpRequest) -> FlashResponse<HttpResponse, String> {
    FlashResponse::with_redirect("This is the message".to_owned(), "/show_flash")
}

fn main() {
    HttpServer::new(move || {
        App::new()
            .wrap(FlashMiddleware::default())
            .route("/show_flash", web::get().to(show_flash))
            .route("/set_flash", web::get().to(set_flash))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .unwrap();
}
