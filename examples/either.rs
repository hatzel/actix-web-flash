use actix_web::{http, server, App, HttpRequest, HttpResponse, Responder, Either};
use actix_web::fs::NamedFile;
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
fn set_flash(req: &HttpRequest) -> Either<impl Responder, impl Responder> {
    if req.query().len() > 1 {
        Either::A(
            FlashResponse::new(
                Some(format!("Query string: {:?}", req.query()).to_owned()),
                HttpResponse::SeeOther()
                    .header(http::header::LOCATION, "/show_flash")
                    .finish(),
            )
        )
    } else {
        Either::B(NamedFile::open("README.md"))
    }
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
