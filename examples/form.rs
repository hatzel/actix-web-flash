use actix_web::{http, server, App, Form, HttpResponse, Responder};
use actix_web_flash::{FlashMessage, FlashMiddleware, FlashResponse};
use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Password {
    password: String,
}

fn get_login(flash: Option<FlashMessage<String>>) -> impl Responder {
    let error = flash
        .map(|i| i.into_inner())
        .unwrap_or_else(|| "".to_owned());
    // Warning; String formatting like this dangerous! Use tera or one of the options listed here
    // instead: https://www.arewewebyet.org/topics/templating/.
    let form = format!(
        r#"<html>
<form class="" action="" method="post">
    <input type="password" name="password" id="password" value="">
    <p style="color:red;">{}</p>
    <input type="submit" value="Login" />
</form>
</html>
    "#,
        error
    );
    let mut resp = HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .finish();
    resp.set_body(form);
    resp
}

fn post_login(form: Form<Password>) -> impl Responder {
    if form.into_inner().password == "hunter2" {
        FlashResponse::new(None, "Correct Password!".into())
    } else {
        FlashResponse::with_redirect("Incorrect Password".to_owned(), "login")
    }
}

fn main() {
    server::new(|| {
        App::new()
            .middleware(FlashMiddleware::default())
            .route("/login", http::Method::GET, get_login)
            .route("/login", http::Method::POST, post_login)
    }).bind("127.0.0.1:8080")
        .unwrap()
        .run();
}
