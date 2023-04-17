use actix_web::{web};

pub fn service() -> actix_web::Scope {
    web::scope("/admin")
}
