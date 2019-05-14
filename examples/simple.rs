use actix_web::{
    middleware, 
    web, 
    App, 
    HttpResponse, 
    HttpServer,
    HttpRequest,
    Responder,
};
use actix_web_flash::{FlashMessage, FlashResponse, FlashMiddleware};

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
        .bind("0.0.0.0:3000")
        .unwrap()
        .run();
}
