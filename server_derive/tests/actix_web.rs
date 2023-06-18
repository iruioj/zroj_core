use actix_web::{HttpResponse, HttpRequest};
use server_derive::scope_service;

async fn default_route(_: HttpRequest) -> HttpResponse {
    HttpResponse::NotFound().finish()
}

#[allow(dead_code)]
#[allow(unsafe_code)]
#[scope_service(path = "/auth", stable = true)]
pub fn service() {
    default_service(actix_web::web::route().to(default_route));
}

#[test]
fn it_works() {}
