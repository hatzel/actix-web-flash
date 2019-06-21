use actix_web::{web, App, HttpResponse, HttpServer, Responder};
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
    let resp = HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(form);
    resp
}

fn post_login(form: web::Form<Password>) -> impl Responder {
    if form.into_inner().password == "hunter2" {
        FlashResponse::new(None, "Correct Password!".into())
    } else {
        FlashResponse::with_redirect("Incorrect Password".to_owned(), "login")
    }
}

fn main() {
    HttpServer::new(|| {
        App::new()
            .wrap(FlashMiddleware::default())
            .route("/login", web::get().to(get_login))
            .route("/login", web::post().to(post_login))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .unwrap();
}
