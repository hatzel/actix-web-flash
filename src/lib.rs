//! Actix web flash is an unofficial crate to provide flash messages in servers using Actix web.
//!
//! Flash messages are typically used to display errors on websites that are rendered server side.
//!
//! A user might post a login form with a password. The server notices the password is incorrect.
//! It has to respond with an error. A common approach is to redirect the client to the same form.
//! The error is displayed by being rendered into the HTML markup on the server side.
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
//! use actix_web::{http, web, HttpServer, App, HttpRequest, HttpResponse, Responder};
//! use actix_web_flash::{FlashMessage, FlashResponse, FlashMiddleware};
//!
//! fn show_flash(flash: FlashMessage<String>) -> impl Responder {
//!     flash.into_inner()
//! }
//!
//! fn set_flash(_req: HttpRequest) -> FlashResponse<HttpResponse, String> {
//!     FlashResponse::new(
//!         Some("This is the message".to_owned()),
//!         HttpResponse::SeeOther()
//!             .header(http::header::LOCATION, "/show_flash")
//!             .finish(),
//!     )
//! }
//!
//! fn main() {
//!     HttpServer::new(|| {
//!         App::new()
//!             .wrap(FlashMiddleware::default())
//!             .route("/show_flash", web::get().to(show_flash))
//!             .route("/set_flash", web::get().to(set_flash))
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
//! Meaning an error message will, if no middleware is present, persist unless replaced by a newer one.
#![deny(missing_docs)]
use actix_service::{Service, Transform};
use actix_web::cookie::{Cookie, CookieJar};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::ErrorBadRequest;
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures::future::{ok as fut_ok, Either as EitherFuture, Future, FutureResult, IntoFuture};
use futures::Poll;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;

#[cfg(test)]
mod tests;

pub(crate) const FLASH_COOKIE_NAME: &str = "_flash";

/// Represents a flash message and implements `actix::FromRequest`
///
/// It is used to retrieve the current flash message from a request and to set a new one in
/// a response.
#[derive(Debug)]
pub struct FlashMessage<T>(T)
where
    T: DeserializeOwned;

impl<T> FromRequest for FlashMessage<T>
where
    T: DeserializeOwned,
{
    type Config = ();
    type Future = Result<FlashMessage<T>, Self::Error>;
    type Error = Error;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
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
    /// Create a new flash message with `inner` as the content.
    pub fn new(inner: M) -> Self {
        FlashMessage(inner)
    }

    /// Retrieve the content of a flash message.
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
    R: Responder + 'static,
    R::Future: 'static,
    M: Serialize + DeserializeOwned + 'static,
{
    type Error = actix_http::Error;
    type Future = Box<dyn Future<Item = HttpResponse, Error = Self::Error>>;

    fn respond_to(mut self, req: &HttpRequest) -> Self::Future {
        let message = self.message.take();

        let out = self
            .delegate_to
            .respond_to(req)
            .into_future()
            .map_err(|e| e.into())
            .and_then(|mut response| {
                if let Some(msg) = message {
                    let data =
                        serde_json::to_string(&msg.into_inner()).expect("Serialize cannot fail");
                    let mut flash_cookie = Cookie::new(FLASH_COOKIE_NAME, data);
                    flash_cookie.set_path("/");
                    let out = response
                        .add_cookie(&flash_cookie)
                        .into_future()
                        .map_err(|e| e.into())
                        .map(|_| response);
                    EitherFuture::A(out)
                } else {
                    EitherFuture::B(futures::future::ok(response))
                }
            });

        Box::new(out)
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
    /// Create a flash response that redirects to a given location.
    /// The response code is `303 - See Other` meaning the client will perform a `GET` request on the
    /// resource you are redirecting to.
    ///
    ///! ```
    ///! # use actix_web_flash::{FlashResponse, FlashMessage};
    ///! # use actix_web::Responder;
    ///! #
    ///! fn show_flash(flash: FlashMessage<String>) -> impl Responder {
    ///!     FlashResponse::with_redirect("Your message".to_owned(), "/show_flash")
    ///! }
    ///
    ///! ```
    ///
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
/// # use actix_web::{App, HttpServer};
/// HttpServer::new(|| {
///     App::new()
///         .wrap(FlashMiddleware::default())
/// }).bind("127.0.0.1:8080")
///     .unwrap()
///     .run();
/// ```
pub struct FlashMiddleware;

impl<S, B> Transform<S> for FlashMiddleware
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = FlashMiddlewareServiceWrapper<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        fut_ok(FlashMiddlewareServiceWrapper(service))
    }
}

/// The actual Flash middleware
pub struct FlashMiddlewareServiceWrapper<S>(S);

impl<S, B> Service for FlashMiddlewareServiceWrapper<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.0.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        Box::new(self.0.call(req).and_then(move |mut res| {
            let mut jar = CookieJar::new();
            if let Some(cookie) = res.request().cookie(FLASH_COOKIE_NAME) {
                jar.add_original(cookie);
                jar.remove(Cookie::named(FLASH_COOKIE_NAME));
            }
            for cookie in jar.delta() {
                res.response_mut().add_cookie(cookie)?;
            }

            Ok(res)
        }))
    }
}
