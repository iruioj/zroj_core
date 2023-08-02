//! ZROJ 后端服务库

pub mod app;
pub mod auth;
pub mod data;
pub mod manager;
pub mod rev_proxy;

pub mod dev;

// pub mod config;
pub type GroupID = u32;
pub type SessionID = uuid::Uuid;
pub type UserID = u32;
pub type ProblemID = u32;

// re-export
pub use actix_session;
use serde_ts_typing::TypeExpr;

/// 可以覆盖 T 类型的默认值
trait Override<T> {
    /// 消耗掉自己并覆盖 T 类型的默认值，
    /// 调用此方法后 self 将不再能被访问
    fn over(self, origin: &mut T);
}

pub(crate) mod marker {
    use actix_web::web;
    /// 标记一个 API 的 body 类型
    pub type JsonBody<T> = web::Json<T>;
    /// 标记从 url query 中获取的数据
    pub type QueryParam<T> = web::Query<T>;
    /// 标记一个 API 的返回数据类型
    pub type JsonResult<T> = actix_web::Result<web::Json<T>>;
    /// 标记一个 API 的返回数据类型为 any
    pub type AnyResult<T> = actix_web::Result<T>;
    /// 标记一个 API 需要用到的服务器数据
    pub type ServerData<T> = web::Data<T>;
    /// 标记一个 API 的 body 类型，使用 [`actix_multipart::form::MultipartForm`] extractor
    pub type FormData<T> = actix_multipart::form::MultipartForm<T>;
}

#[derive(Debug)]
pub struct ApiDocMeta {
    pub path: String,
    pub method: String,
    pub query_type: Option<TypeExpr>,
    pub body_type: Option<TypeExpr>,
    pub is_form: bool,
    pub res_type: Option<TypeExpr>,
    pub description: String,
}

#[derive(Debug)]
pub struct ServiceDoc {
    pub path: String,
    pub apis: Vec<ApiDocMeta>,
}
