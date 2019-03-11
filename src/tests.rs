use crate::{FlashMessage, FlashResponse, Responder, FlashMiddleware};
use actix_web::{FromRequest, HttpResponse, App, http, HttpRequest};
use actix_web::http::{Cookie, StatusCode};
use actix_web::test::{TestRequest, TestServer};

#[test]
/// Ensure the response properly sets the `_flash` cookie.
fn sets_cookie() {
    let msg = "Test Message".to_owned();
    let response = FlashResponse::new(Some(msg.clone()), HttpResponse::Ok().finish());
    let req = TestRequest::default()
        .execute(|req| response.respond_to(req))
        .unwrap();
    let cookies = req.cookies()
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
    let req = TestRequest::with_header("Cookie", "_flash=\"Test Message\"").finish();
    let msg = FlashMessage::<String>::from_request(&req, &()).unwrap();
    assert_eq!(msg.into_inner(), "Test Message");
}

#[test]
/// Ensure improper cookie contents lead to an error.
fn bad_request() {
    let req = TestRequest::with_header("Cookie", "_flash=Missing quotes").finish();
    let err = FlashMessage::<String>::from_request(&req, &()).unwrap_err();
    // Don't return raw serialization errors
    assert!(err.downcast_ref::<serde_json::error::Error>().is_none());
    let resp = HttpResponse::from(err);
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST)
}

fn minimal_app() -> App {
    App::new()
        .middleware(FlashMiddleware::default())
        .route("/show_flash", http::Method::GET, show_flash)
        .resource("/set_flash", |r| r.f(set_flash))
}

fn show_flash(flash: FlashMessage<String>) -> impl Responder {
    flash.into_inner()
}

fn set_flash(_req: &HttpRequest) -> FlashResponse<HttpResponse, String> {
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
    let mut srv = TestServer::with_factory(minimal_app);

    let req = srv.client(http::Method::GET, "/set_flash").finish().unwrap();
    let resp = srv.execute(req.send()).unwrap();
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);

    let cookie = resp.cookie("_flash").unwrap();
    assert_eq!(cookie.value(), "\"This is the message\"");
}

#[test]
/// Integration test to assure the cookie is deleted on request
fn cookie_is_unset() {
    let mut srv = TestServer::with_factory(minimal_app);
    let req = srv.get().cookie(
            http::Cookie::build("_flash", "To be deleted")
            .path("/")
            .finish()
        ).finish().unwrap();
    let resp = srv.execute(req.send()).unwrap();
    println!("{:?}", resp);
    let cookie = resp.cookie("_flash").unwrap();
    println!("Cookie: {:?}", cookie);
    assert!(cookie.expires().unwrap() < time::now());
}
