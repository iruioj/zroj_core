use actix_web::{get, post, web, HttpResponse, Responder, Error};

#[get("/global_config")]
async fn global_config() -> impl Responder {
    HttpResponse::Ok().body(format!("Server configuration"))
}
#[get("/judge_actions")]
async fn judge_actions() -> impl Responder {
    HttpResponse::Ok().body(format!("Watch judge list"))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg .service(global_config)
        .service(judge_actions);
}



