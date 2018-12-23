use actix_web::{Error, FromRequest, HttpRequest, HttpResponse, Responder};
use actix_web::http::Cookie;
use actix_web::error::ErrorBadRequest;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_derive;
use serde_json;
use futures::future::Future;

const FLASH_COOKIE_NAME: &str = "_flash";

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
            // TODO: this ? might not work too well
            let inner = serde_json::from_str(cookie.value())?;
            Ok(FlashMessage(inner))
        } else {
            Err(ErrorBadRequest("No Flash Cookie."))
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
    message: FlashMessage<M>,
}

impl<R, M> Responder for FlashResponse<R, M>
where
    R: Responder,
    M: Serialize + DeserializeOwned,
{
    type Item = HttpResponse;
    type Error = Error;

    fn respond_to<S: 'static>(self, req: &HttpRequest<S>) -> Result<Self::Item, Self::Error> {
        let data = serde_json::to_string(&self.message.into_inner())?;
        let flash_cookie = Cookie::new(FLASH_COOKIE_NAME, data);
        let response = self.delegate_to
            .respond_to(req)
            .map(|v| v.into())
            .map_err(|v| v.into())?;
        let response_future = response.then(|res| -> Result<_, Error> {
            res.and_then(|mut req| {
                req.add_cookie(&flash_cookie)
                    .map_err(|e| e.into())
                    .map(|_| req)
            })
        });
        response_future.wait().into()
    }
}

impl<R, M> FlashResponse<R, M>
where
    R: Responder,
    M: Serialize + DeserializeOwned,
{
    pub fn flash(response: R, message: FlashMessage<M>) -> Self {
        Self {
            delegate_to: response,
            message: message,
        }
    }
}
