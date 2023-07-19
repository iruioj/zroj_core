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
pub type ProblemID = problem::database::ProbID;

// re-export
pub use actix_session;

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
    /// 标记一个 API 需要用到的服务器数据
    pub type ServerData<T> = web::Data<T>;
}

#[derive(Debug)]
pub struct ApiDocMeta {
    pub path: String,
    pub method: String,
    pub query_type: Option<String>,
    pub body_type: Option<String>,
    pub res_type: Option<String>,
    pub description: String,
}

#[derive(Debug)]
pub struct ServiceDoc {
    pub path: String,
    pub apis: Vec<ApiDocMeta>,
}
