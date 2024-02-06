//! http backend services provided by online judge server
pub mod auth;
// pub mod group;
pub mod api_docs;
pub mod one_off;
pub mod problem;
pub mod submission;
pub mod user;

use judger::SourceFile;
use serde::Deserialize;
use serde_ts_typing::TsType;
use serde_with::{serde_as, DisplayFromStr};

/// 默认 404
// pub async fn default_route(req: HttpRequest) -> HttpResponse {
//     let mut r = String::new();
//     r.push_str("Not found\n\n");
//     r.push_str(format!("Uri: {}\n", req.uri()).as_str());
//     r.push_str(format!("Method: {}\n", req.method()).as_str());
//     r.push_str("Headers:\n");
//     for (name, val) in req.headers() {
//         r.push_str(format!("- {}:{:?}\n", name, val).as_str());
//     }
//     HttpResponse::NotFound().body(r)
// }

/// 将命名格式为 `name.type.suffix` 的文件解析为 SourceFile
/// 主要用于 one_off 和 problem::submit 解析 payload
fn parse_named_file(
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

// ref: https://docs.rs/serde_qs/0.12.0/serde_qs/index.html#flatten-workaround
#[serde_as]
#[derive(Deserialize, TsType)]
#[ts(inline)]
struct ListQuery {
    #[serde_as(as = "DisplayFromStr")]
    // TypeScript 类型仍然为 u8，因为通过 x-www-form-urlencoded 格式传递
    #[ts(as_type = "u8")]
    /// 利用类型限制，一次请求的数量不能超过 256 个
    max_count: u8,

    #[serde_as(as = "DisplayFromStr")]
    // TypeScript 类型仍然为 u32，因为通过 x-www-form-urlencoded 格式传递
    #[ts(as_type = "u32")]
    /// 跳过前 offset 个结果
    offset: u32,
}
