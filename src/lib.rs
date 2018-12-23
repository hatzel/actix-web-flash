//! Actix web flash is an unofficial crate to provide flash messages in servers using Actix web.
//!
//! Flash messages are typically to display errors on in websites that are rendered server side.
//!
//! A user might post a login form with their user name and password.
//! The server notices the password is incorrect.
//! It has to respond with an error. A common approach is to redirect the client to the same form.
//!
//! ```
//! use actix_web::{http, server, App, HttpRequest, HttpResponse, Responder};
//! use actix_web_flash::{FlashMessage, FlashResponse};
//!
//! fn show_flash(flash: FlashMessage<String>) -> impl Responder {
//!     flash.into_inner()
//! }
//!
//! fn set_flash(_req: &HttpRequest) -> FlashResponse<HttpResponse, String> {
//!     FlashResponse::new(
//!         Some("This is the message".to_owned()),
//!         HttpResponse::SeeOther()
//!             .header(http::header::LOCATION, "/show_flash")
//!             .finish(),
//!     )
//! }
//!
//! fn main() {
//!     server::new(|| {
//!         App::new()
//!             .route("/show_flash", http::Method::GET, show_flash)
//!             .resource("/set_flash", |r| r.f(set_flash))
//!     }).bind("127.0.0.1:8080")
//!         .unwrap()
//!         .run();
//! }
//! ```
//!
//! The data is relayed to the next request via a cookie. This means its not suitable for large data!
//! Currently `actix-web-flash` does not implement any cryptographic checks of the cookie's
//! validity. Treat it as untrusted input!
use actix_web::{Error, FromRequest, HttpRequest, HttpResponse, Responder};
use actix_web::http::Cookie;
use actix_web::error::ErrorBadRequest;
use actix_web::middleware::{Middleware, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_derive;
use serde_json;
use time;
use futures::future::Future;

#[cfg(test)]
mod tests;

pub(crate) const FLASH_COOKIE_NAME: &str = "_flash";

#[derive(Debug)]
pub struct FlashMessage<T>(T)
where
    T: DeserializeOwned;

impl<S, T> FromRequest<S> for FlashMessage<T>
where
    T: DeserializeOwned,
{
    type Config = ();
    type Result = Result<FlashMessage<T>, Error>;

    fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        if let Some(cookie) = req.cookie(FLASH_COOKIE_NAME) {
            let inner = serde_json::from_str(cookie.value())
                .map_err(|_| ErrorBadRequest("Invalid flash cookie"))?;
            Ok(FlashMessage(inner))
        } else {
            Err(ErrorBadRequest("No flash cookie"))
        }
    }
}

impl<M> FlashMessage<M>
where
    M: Serialize + DeserializeOwned,
{
    pub fn new(inner: M) -> Self {
        FlashMessage(inner)
    }

    pub fn into_inner(self) -> M {
        self.0
    }
}

pub struct FlashResponse<R, M>
where
    R: Responder,
    M: Serialize + DeserializeOwned,
{
    delegate_to: R,
    message: Option<FlashMessage<M>>,
}

impl<R, M> Responder for FlashResponse<R, M>
where
    R: Responder,
    M: Serialize + DeserializeOwned,
{
    type Item = HttpResponse;
    type Error = Error;

    fn respond_to<S: 'static>(self, req: &HttpRequest<S>) -> Result<Self::Item, Self::Error> {
        let response = self.delegate_to
            .respond_to(req)
            .map(|v| v.into())
            .map_err(|v| v.into())?;
        if let Some(msg) = self.message {
            let data = serde_json::to_string(&msg.into_inner())?;
            let flash_cookie = Cookie::new(FLASH_COOKIE_NAME, data);
            let response_future = response.then(|res| -> Result<_, Error> {
                res.and_then(|mut req| {
                    req.add_cookie(&flash_cookie)
                        .map_err(|e| e.into())
                        .map(|_| req)
                })
            });
            response_future.wait()
        } else {
            response.wait()
        }
    }
}

impl<R, M> FlashResponse<R, M>
where
    R: Responder,
    M: Serialize + DeserializeOwned,
{
    ///
    /// Constructs a new `FlashResponse` with a desired message and response.
    ///
    /// The message is saved in a cookie and can be extracted on the next request.
    /// In a typical use-case for the response would be a redirect to a page that will display the
    /// message.
    ///
    /// ```
    /// # use actix_web_flash::{FlashMessage, FlashResponse};
    /// #
    /// FlashResponse::new(
    ///     Some("Some error occurred".to_owned()),
    ///     actix_web::HttpResponse::Ok()
    ///         .header(actix_web::http::header::LOCATION, "/render_error")
    ///         .finish(),
    /// );
    /// ```
    pub fn new(message: Option<M>, response: R) -> Self {
        Self {
            delegate_to: response,
            message: message.map(|m| FlashMessage(m)),
        }
    }
}

#[derive(Debug, Default)]
/// `FlashMiddleware` takes care of deleting the flash cookies after their use.
///
/// Without this middle ware any flash message is be passed into all handlers requesting it, until the cookie
/// is overwritten by a new message.
/// ```
/// server::new(|| {
///     App::new()
///         .middleware(FlashMiddleware::default())
/// }).bind("127.0.0.1:8080")
///     .unwrap()
///     .run();
/// ```
pub struct FlashMiddleware();

impl<S> Middleware<S> for FlashMiddleware {
    fn response(&self, req: &HttpRequest<S>, mut resp: HttpResponse) -> Result<Response, actix_web::Error> {
        let received_flash = req.cookie(FLASH_COOKIE_NAME);
        if received_flash.is_some() && resp.cookies().find(|ref c| c.name() == FLASH_COOKIE_NAME).is_none() {
            // Delete cookie by setting an expiry date in the past
            let mut expired = Cookie::new(FLASH_COOKIE_NAME, "");
            let time = time::strptime("1970-1-1", "%Y-%m-%d").unwrap();
            expired.set_expires(time);
            resp.add_cookie(&expired)?;
        }
        Ok(Response::Done(resp))
    }
}
