# Actix Web Flash
[![Build Status](https://travis-ci.org/hatzel/actix-web-flash.svg?branch=master)](https://travis-ci.org/hatzel/actix-web-flash)
[![Crates.io](https://img.shields.io/crates/v/actix-web-flash.svg)](https://crates.io/crates/actix-web-flash)
[![dependency status](https://deps.rs/repo/github/hatzel/actix-web-flash/status.svg)](https://deps.rs/repo/github/hatzel/actix-web-flash)
[![license](https://img.shields.io/crates/l/actix-web-flash.svg)](https://github.com/hatzel/actix-web-flash/blob/master/LICENSE-MIT)

Actix web flash is an unofficial crate to provide flash messages in servers using Actix web.

Flash messages are typically used to display errors on websites that are rendered server side.

* [Docs](https://docs.rs/actix-web-flash/0.1.0/actix-web-flash/)
* [Examples](examples/)

```rust
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash::{FlashMessage, FlashMiddleware, FlashResponse};

fn show_flash(flash: FlashMessage<String>) -> impl Responder {
    flash.into_inner()
}

fn set_flash(_req: HttpRequest) -> FlashResponse<HttpResponse, String> {
    FlashResponse::with_redirect("This is the message".to_owned(), "/show_flash")
}

fn main() {
    HttpServer::new(move || {
        App::new()
            .wrap(FlashMiddleware::default())
            .route("/show_flash", web::get().to(show_flash))
            .route("/set_flash", web::get().to(set_flash))
    }).bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .unwrap();
}
```

## License

MIT/Apache-2.0
