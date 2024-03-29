//! HTTP backend application services provided by online judge server
pub mod auth;
// pub mod group;
pub mod api_docs;
pub mod contest;
pub mod one_off;
pub mod problem;
pub mod submission;
pub mod user;

use judger::SourceFile;
use serde::Deserialize;
use serde_ts_typing::TsType;
use serde_with::{serde_as, DisplayFromStr};

use crate::ServiceDoc;

/// 将命名格式为 `name.type.suffix` 的文件解析为 SourceFile
/// 主要用于 one_off 和 problem::submit 解析 payload
pub fn parse_named_file(
    nf: &actix_multipart::form::tempfile::TempFile,
) -> Option<(String, SourceFile)> {
    let binding = nf.file_name.as_ref()?;
    let lang: Vec<&str> = binding.trim().split('.').collect();
    let name = lang.first()?;
    let ty = lang.get(1)?;
    let file_type: judger::FileType = serde_json::from_value(serde_json::json!(ty)).ok()?;
    tracing::info!("parse_named_file name = {name}, type = {file_type:?}");
    let source = std::io::read_to_string(nf.file.reopen().ok()?).ok()?;
    Some((name.to_string(), SourceFile { source, file_type }))
}

/// An abstraction of list query. For performance & security reasons, the number
/// of returned items is limited by `max_count`, which is a `u8` value (< 256).
/// The `offset` is used to skip the first `offset` items.
///
/// Every API involving `ListQuery` should return exact `max_count` items unless
/// there's not enough items starting from this offset. Under this condition, one
/// can implement paginated query conveniently.
// ref: https://docs.rs/serde_qs/0.12.0/serde_qs/index.html#flatten-workaround
#[serde_as]
#[derive(Deserialize, TsType)]
#[ts(inline)]
struct ListQuery {
    /// restricted automatically by [`u8::MAX`].
    #[serde_as(as = "DisplayFromStr")]
    // TypeScript 类型仍然为 u8，因为通过 x-www-form-urlencoded 格式传递
    #[ts(as_type = "u8")]
    max_count: u8,

    /// skip the first `offset` items.
    #[serde_as(as = "DisplayFromStr")]
    // TypeScript 类型仍然为 u32，因为通过 x-www-form-urlencoded 格式传递
    #[ts(as_type = "u32")]
    offset: u32,
}

lazy_static::lazy_static!(
    pub static ref DOCS: Vec<(ServiceDoc, serde_ts_typing::Context)> = vec![
        auth::service_doc(),
        one_off::service_doc(),
        problem::service_doc(),
        submission::service_doc(),
        user::service_doc(),
        contest::service_doc(),
    ];
);
