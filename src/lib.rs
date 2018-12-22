use actix_web::{FromRequest, Error, HttpRequest};
use actix_web::error::ErrorBadRequest;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_derive::Deserialize;
use serde_derive;
use serde_json;

const FLASH_COOKIE_NAME: &'static str = "_flash";

#[derive(Deserialize)]
struct Msg(String);

#[derive(Debug)]
pub struct Flash<T>(T) where T: DeserializeOwned + Serialize;

// impl<T> Flash<T> where T: DeserializeOwned + Serialize {
//     pub fn new(inner: T) -> Self {
//         Flash(inner)
//     }
// }


impl<S, T> FromRequest<S> for Flash<T> where T: DeserializeOwned + Serialize {
    type Config = ();
    type Result = Result<Flash<T>, Error>;

    fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        if let Some(cookie) = req.cookie(FLASH_COOKIE_NAME) {
            // TODO: this ? might not work too well
            let inner = serde_json::from_str(cookie.value())?;
            Ok(Flash(inner))
        } else {
            Err(ErrorBadRequest("No Flash Cookie."))
        }
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
