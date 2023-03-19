use actix_web::{get, web, HttpResponse, Responder};
use actix_session::{Session};

#[derive(serde::Serialize, Clone, Debug)]
struct ProblemData {
    /// statement should be a json parsed from multiple user input parts
    statement: String,
}

#[get("/{pid}")]
async fn index(pid: web::Path<u32>, session: Session) -> impl Responder {
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
