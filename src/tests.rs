use crate::{FlashMessage, FlashMiddleware, FlashResponse, Responder};
use actix_http::HttpService;
use actix_http_test::TestServer;
use actix_web::http::{Cookie, StatusCode};
use actix_web::test::{self, TestRequest};
use actix_web::{http, web, App, FromRequest, HttpRequest, HttpResponse};

#[test]
/// Ensure the response properly sets the `_flash` cookie.
fn sets_cookie() {
    let msg = "Test Message".to_owned();
    let responder = FlashResponse::new(Some(msg.clone()), HttpResponse::Ok().finish());

    let req = TestRequest::default().to_http_request();

    let resp = test::block_on(responder.respond_to(&req)).unwrap();

    let cookies = resp
        .cookies()
        .filter(|c| c.name() == crate::FLASH_COOKIE_NAME)
        .collect::<Vec<Cookie>>();
    assert_eq!(cookies.len(), 1);
    assert_eq!(cookies[0].name(), crate::FLASH_COOKIE_NAME);
    // JSON serialization means the string is in quotes
    assert_eq!(cookies[0].value(), format!("\"{}\"", msg));
}

#[test]
/// Ensure flash message is extracted from `_flash` cookie.
fn get_cookie() {
    let req = TestRequest::with_header("Cookie", "_flash=\"Test Message\"").to_http_request();
    let msg = FlashMessage::<String>::extract(&req).unwrap();
    assert_eq!(msg.into_inner(), "Test Message");
}

#[test]
/// Ensure improper cookie contents lead to an error.
fn bad_request() {
    let req = TestRequest::with_header("Cookie", "_flash=Missing quotes").to_http_request();
    let err = FlashMessage::<String>::extract(&req).unwrap_err();
    // Don't return raw serialization errors
    assert!(std::error::Error::downcast_ref::<serde_json::error::Error>(&err).is_none());
    let resp = HttpResponse::from(err);
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST)
}

fn minimal_app() -> actix_http_test::TestServerRuntime {
    TestServer::new(|| {
        HttpService::new(
            App::new()
                .wrap(FlashMiddleware::default())
                .route("/show_flash", web::get().to(show_flash))
                .route("/set_flash", web::get().to(set_flash)),
        )
    })
}

fn show_flash(flash: FlashMessage<String>) -> impl Responder {
    flash.into_inner()
}

fn set_flash(_req: HttpRequest) -> FlashResponse<HttpResponse, String> {
    FlashResponse::new(
        Some("This is the message".to_owned()),
        HttpResponse::SeeOther()
            .header(http::header::LOCATION, "/show_flash")
            .finish(),
    )
}

#[test]
/// Integration test to assure the cookie is deleted on request
fn cookie_is_set() {
    use actix_http::httpmessage::HttpMessage;

    let mut srv = minimal_app();

    let req = srv.get("/set_flash");
    let resp = srv.block_on(req.send()).unwrap();
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);

    let cookie = resp.cookie("_flash").unwrap();
    assert_eq!(cookie.value(), "\"This is the message\"");
}

#[test]
/// Integration test to assure the cookie is deleted on request
fn cookie_is_unset() {
    use actix_http::httpmessage::HttpMessage;

    let mut srv = minimal_app();
    let req = srv.get("/").cookie(
        http::Cookie::build("_flash", "To be deleted")
            .path("/")
            .finish(),
    );
    let resp = srv.block_on(req.send()).unwrap();
    println!("{:?}", resp);
    let cookie = resp.cookie("_flash").unwrap();
    println!("Cookie: {:?}", cookie);
    assert!(cookie.expires().unwrap() < time::now());
}
