use actix_web::{get, web, HttpResponse, Responder};

#[get("/global_config")]
async fn global_config() -> impl Responder {
    HttpResponse::Ok().body(format!("Server configuration"))
}
#[get("/judge_actions")]
async fn judge_actions() -> impl Responder {
    HttpResponse::Ok().body(format!("Watch judge list"))
}

pub fn service() -> actix_web::Scope {
    web::scope("/admin")
        .service(global_config)
        .service(judge_actions)
}
