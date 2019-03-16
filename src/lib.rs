//! Actix web flash is an unofficial crate to provide flash messages in servers using Actix web.
//!
//! Flash messages are typically used to display errors on websites that are rendered server side.
//!
//! A user might post a login form with a password. The server notices the password is incorrect.
//! It has to respond with an error. A common approach is to redirect the client to the same form.
//! The error is displayed by being rendered into the html markup on the server side.
//!
//! The data is relayed to the next request via a cookie. This means its not suitable for large data!
//! Currently `actix-web-flash` does not implement any cryptographic checks of the cookie's
//! validity. Treat it as untrusted input!
//!
//! You can find example code below and in the
//! [repository](https://github.com/hatzel/actix-web-flash/tree/master/examples).
//!
//! ## Trivial Example
//! ```no_run
//! use actix_web::{http, server, App, HttpRequest, HttpResponse, Responder};
//! use actix_web_flash::{FlashMessage, FlashResponse, FlashMiddleware};
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
//!             .middleware(FlashMiddleware::default())
//!             .route("/show_flash", http::Method::GET, show_flash)
//!             .resource("/set_flash", |r| r.f(set_flash))
//!     }).bind("127.0.0.1:8080")
//!         .unwrap()
//!         .run();
//! }
//! ```
//! The example above will redirect the user from `/set_flash` to `/show_flash` and pass along
//! a string, rendering it right away.
//!
//! ## Arbitrary Types
//! Arbitrary types can be used as flash messages.
//!
//! ```
//! use actix_web::Responder;
//! use actix_web_flash::FlashMessage;
//! use serde_derive::{Serialize, Deserialize};
//!
//! #[derive(Deserialize, Serialize, Debug)]
//! struct MyData {
//!     msg: String,
//!     color: (u8, u8, u8),
//! }
//!
//! fn show_flash(flash: FlashMessage<MyData>) -> impl Responder {
//!     format!("Message {:?}", flash.into_inner())
//! }
//! ```
//!
//! ## Optional Messages
//! It is possible to take an `Option<FlashMessage<T>>` thereby allowing for your route to also be
//! called without a flash message having been returned in the previous request.
//!
//! ```
//! use actix_web::Responder;
//! use actix_web_flash::FlashMessage;
//! use serde_derive::Deserialize;
//!
//!
//! fn show_flash(flash: Option<FlashMessage<String>>) -> impl Responder {
//!     match flash {
//!         Some(msg) => format!("There was some error: {}", msg.into_inner()),
//!         None => "All is good!".to_owned()
//!     }
//! }
//! ```
//!
//! ## Limitations and Pitfalls
//!
//! Only a single message is supported. It can however be of any type (that implements `Deserialize`), meaning a `Vec<YourType>`
//! is possible.
//!
//! The cookie will not be cleared unless the [middleware](actix_web_flash::FlashMiddleware) is registered.
//! Meaning an error message will persist unless replaced with a newer one.
use actix_web::{Error, FromRequest, HttpRequest, HttpResponse, Responder};
use cookie::{Cookie, CookieJar};
use actix_web::error::ErrorBadRequest;
use actix_web::middleware::{Middleware, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json;
use futures::future::Future;

#[cfg(test)]
mod tests;

pub(crate) const FLASH_COOKIE_NAME: &str = "_flash";

/// Represents a flash message and implements `actix::FromRequest`
///
/// It is used to retrieve the currently set flash message.
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

/// Actix response type that sets a flash message
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
    type Item = actix_web::dev::AsyncResult<HttpResponse>;
    type Error = Error;

    fn respond_to<S: 'static>(self, req: &HttpRequest<S>) -> Result<Self::Item, Self::Error> {
        let response = self.delegate_to
            .respond_to(req)
            .map(|v| v.into())
            .map_err(|v| v.into())?;

        if let Some(msg) = self.message {
            let data = serde_json::to_string(&msg.into_inner())?;

            let mut flash_cookie = Cookie::new(FLASH_COOKIE_NAME, data);
            flash_cookie.set_path("/");

            let response_future = response.and_then(move |mut res| {
                res.add_cookie(&flash_cookie)
                    .map_err(|e| e.into())
                    .map(|_| res)
            });
            Ok(actix_web::dev::AsyncResult::future(Box::new(
                response_future,
            )))
        } else {
            Ok(response)
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
            message: message.map(FlashMessage),
        }
    }
}

impl<M> FlashResponse<HttpResponse, M>
where
    M: Serialize + DeserializeOwned,
{
    pub fn with_redirect(message: M, location: &str) -> Self {
        let response = actix_web::HttpResponse::SeeOther()
            .header(actix_web::http::header::LOCATION, location)
            .finish();
        Self::new(Some(message), response)
    }
}

#[derive(Debug, Default)]
/// Takes care of deleting the flash cookies after their use.
///
/// Without this middle ware any flash message is be passed into all handlers requesting it, until
/// the cookie is overwritten by a new message.
/// ```no_run
/// # use actix_web_flash::{FlashMiddleware};
/// # use actix_web::{App, server};
/// server::new(|| {
///     App::new()
///         .middleware(FlashMiddleware::default())
/// }).bind("127.0.0.1:8080")
///     .unwrap()
///     .run();
/// ```
pub struct FlashMiddleware();

impl<S> Middleware<S> for FlashMiddleware {
    fn response(
        &self,
        req: &HttpRequest<S>,
        mut resp: HttpResponse,
    ) -> Result<Response, actix_web::Error> {
        let mut jar = CookieJar::new();
        if let Some(cookie) = req.cookie(FLASH_COOKIE_NAME) {
            jar.add_original(cookie);
            jar.remove(Cookie::named(FLASH_COOKIE_NAME));
        }
        for cookie in jar.delta() {
            resp.add_cookie(cookie)?;
        }
        Ok(Response::Done(resp))
    }
}
