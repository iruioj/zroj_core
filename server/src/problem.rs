use actix_web::{get, post, web, HttpResponse, Responder, Error};

#[get("/{pid}")]
async fn index(pid: web::Path<u32>) -> impl Responder {
    HttpResponse::Ok().body(format!("Problem Index Page pid: {}", pid))
}
#[get("/{pid}/edit")]
async fn editpage(pid: web::Path<u32>) -> impl Responder {
    HttpResponse::Ok().body(format!("Edit Problem pid: {}", pid))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg .service(index)
        .service(editpage);
}
