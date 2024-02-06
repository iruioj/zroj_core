#![doc = include_str!("../README.md")]

pub mod app;
pub mod auth;
pub mod data;
pub mod manager;
pub mod rev_proxy;

pub mod dev;

use rustls::{ClientConfig, RootCertStore};

// pub mod config;
pub type GroupID = u32;
pub type ClientID = uuid::Uuid;
pub type UserID = u32;
pub type ProblemID = u32;
pub type SubmID = u32;

// re-export
#[cfg(session_auth)]
pub use actix_session;
use serde_ts_typing::TypeExpr;

use problem::Override;

/// marker are used with the [`server_derive::api`] macro.
pub(crate) mod marker {
    use actix_web::web;
    /// 标记一个 API 的 body 类型
    pub type JsonBody<T> = web::Json<T>;
    /// 标记从 url query 中获取的数据
    pub type QueryParam<T> = web::Query<T>;
    /// 标记一个 API 的返回数据类型
    pub type JsonResult<T> = actix_web::Result<web::Json<T>>;
    /// 标记一个 API 的返回数据类型为 T
    pub type AnyResult<T> = actix_web::Result<T>;
    /// 标记一个 API 需要用到的服务器数据
    pub type ServerData<T> = web::Data<T>;
    /// 标记一个 API 的 body 类型，使用 [`actix_multipart::form::MultipartForm`] extractor
    pub type FormData<T> = actix_multipart::form::MultipartForm<T>;
}

/// Convenient shortcut for [`actix_web::web::block`], which executes blocking
/// function on a thread pool, returns future that resolves to result
/// of the function execution.
#[macro_export]
macro_rules! block_it {
    {$( $line:stmt );*} => {
        actix_web::web::block(move || { $( $line );* }).await?
    };
}

/// The returning value of api document metadata generator.
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

/// The returning value of service document metadata generator.
#[derive(Debug)]
pub struct ServiceDoc {
    pub path: String,
    pub apis: Vec<ApiDocMeta>,
}

/// Create simple rustls client config from root certificates.
pub fn rustls_config() -> ClientConfig {
    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth()
}
