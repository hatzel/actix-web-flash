use actix_files::NamedFile;
use actix_web::{web, App, Either, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash::{FlashMessage, FlashMiddleware, FlashResponse};

fn show_flash(flash: FlashMessage<String>) -> impl Responder {
    flash.into_inner()
}

/// When using flash messages you often find yourself retuning different types in the error and non
/// error cases. actix-web offers the `Either` type to help you do this.
///
/// Unfortunately you can not return arbitrary types implementing `Responder`.
/// Using returning a boxed trait object will no work due to `respond_to` (a method of `Responder`)
/// being parameterized via monomorphisation as opposed to dynamic dispatch.
fn set_flash(req: HttpRequest) -> Either<impl Responder, impl Responder> {
    if req.query_string().len() > 1 {
        Either::A(FlashResponse::new(
            Some(format!("Query string: {}", req.query_string()).to_owned()),
            HttpResponse::SeeOther()
                .header(actix_http::http::header::LOCATION, "/show_flash")
                .finish(),
        ))
    } else {
        Either::B(NamedFile::open("README.md"))
    }
}

fn main() {
    HttpServer::new(|| {
        App::new()
            .wrap(FlashMiddleware::default())
            .route("/show_flash", web::get().to(show_flash))
            .route("/set_flash", web::route().to(set_flash))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .unwrap();
}
