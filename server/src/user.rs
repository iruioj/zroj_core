use actix_web::{get,post,web, HttpResponse, Responder, Error};

#[get("/login")]
async fn login() -> impl Responder {
    // should check session info
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(include_str!("static/login.html"))
}

#[get("/user/{uid}")]
async fn user_index(uid : web::Path <i32>) -> impl Responder {
    // should check user privilege
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(include_str!("static/user_index.html"))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg .service(login)
        .service(user_index);
}

