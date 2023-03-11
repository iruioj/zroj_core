use actix_web::{get, web, HttpResponse, Responder};

#[get("/{pid}")]
async fn index(pid: web::Path<u32>) -> impl Responder {
    HttpResponse::Ok().body(format!("Problem Index Page pid: {}", pid))
}

#[get("/{pid}/edit")]
async fn editpage(pid: web::Path<u32>) -> impl Responder {
    HttpResponse::Ok().body(format!("Edit Problem pid: {}", pid))
}

/// 提供 problem 的网络服务
pub fn service() -> actix_web::Scope {
    web::scope("/problem").service(index).service(editpage)
}
