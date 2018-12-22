use actix_web::{FromRequest, Error, HttpRequest, Responder, HttpResponse};
use actix_web::dev::AsyncResult;
use actix_web::error::ErrorBadRequest;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_derive::Deserialize;
use serde_derive;
use serde_json;

const FLASH_COOKIE_NAME: &str = "_flash";

#[derive(Deserialize)]
struct Msg(String);

#[derive(Debug)]
pub struct FlashMessage<T>(T) where T: DeserializeOwned + Serialize;

// TODO: consider removing Serialize contraint here
impl<S, T> FromRequest<S> for FlashMessage<T> where T: DeserializeOwned + Serialize {
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


struct FlashResponse<R, M> where R: Responder, M: Serialize + DeserializeOwned {
    delegate_to: R,
    message: FlashMessage<M>
}

impl<R, M> Responder for FlashResponse<R, M> where R: Responder, M: Serialize + DeserializeOwned {
    type Item = AsyncResult<HttpResponse>;
    type Error = Error;

    fn respond_to<S: 'static>(self, req: &HttpRequest<S>) -> Result<Self::Item, Self::Error> {
        self.delegate_to.respond_to(req).map(|v| v.into()).map_err(|v| v.into())
    }
}

impl<R, M> FlashResponse<R, M> where R: Responder, M: Serialize + DeserializeOwned {
    fn flash(response: R, message: FlashMessage<M>) {
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

// TODO: middelware for removing _flash cookie: Just always remove unless current request is
// setting it
