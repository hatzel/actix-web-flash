use crate::{FlashMessage, FlashResponse, Responder};
use actix_web::{FromRequest, HttpResponse};
use actix_web::http::Cookie;
use actix_web::test::TestRequest;

#[test]
/// Ensure the response properly sets the `_flash` cookie.
fn sets_cookie() {
    let msg = "Test Message".to_owned();
    let response =
        FlashResponse::flash(HttpResponse::Ok().finish(), FlashMessage::new(msg.clone()));
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
