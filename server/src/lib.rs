//! ZROJ 后端服务库

pub mod app;
pub mod auth;
pub mod config;
pub mod data;
pub mod manager;
mod problem;

pub type GroupID = u32;
pub type SessionID = uuid::Uuid;
pub type UserID = u32;
pub type ProblemID = u32;

// re-export
pub use actix_session;

/// 可以覆盖 T 类型的默认值
trait Override<T> {
    /// 消耗掉自己并覆盖 T 类型的默认值，
    /// 调用此方法后 self 将不再能被访问
    fn over(self, origin: &mut T);
}
