use actix_web::{get, post, web::{self, get}, HttpResponse, Responder, Error};


#[post("/register")]
async fn register() -> impl Responder {
    // should return json
    HttpResponse::Ok().body(format!("api: Register Success"))
}
#[post("/login")]
async fn login() -> impl Responder {
    HttpResponse::Ok().body(format!("api: Login Success"))
}

// ?action=addtional_file
// ?action=test_data&filename={filename}
#[get("get_data/{pid}")] 
async fn get_data(pid: web::Path<u32>) -> impl Responder {
    HttpResponse::Ok().body(format!("api: GET problem data pid: {}", pid))
}

// ?action=addtional_file
// ?action=test_data&filename={filename}
// body = data(bytes)
#[post("post_data/{pid}")]
async fn post_data(pid: web::Path<u32>) -> impl Responder {
    HttpResponse::Ok().body(format!("api: POST problem data pid: {}", pid))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg .service(register)
        .service(login)
        .service(get_data)
        .service(post_data) ;
}



