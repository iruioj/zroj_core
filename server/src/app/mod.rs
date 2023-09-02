//! app 模块可以创建 OJ 后端的应用路由配置.
pub mod auth;
// pub mod group;
pub mod one_off;
pub mod problem;
pub mod submission;
pub mod user;

use judger::StoreFile;
use serde::Deserialize;
use serde_ts_typing::TsType;

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

/// 将命名格式为 `name.type.suffix` 的文件解析为 StoreFile
/// 主要用于 one_off 和 problem::submit 解析 payload
fn parse_named_file(nf: &actix_multipart::form::tempfile::TempFile) -> Option<(String, StoreFile)> {
    let binding = nf.file_name.as_ref()?;
    let lang: Vec<&str> = binding.trim().split('.').collect();
    let name = lang.get(0)?;
    let ty = lang.get(1)?;
    let file_type: judger::FileType = serde_json::from_value(serde_json::json!(ty)).ok()?;
    tracing::info!("parse_named_file name = {name}, type = {file_type:?}");
    Some((
        name.to_string(),
        StoreFile {
            file: nf.file.reopen().ok()?,
            file_type,
        },
    ))
}

/// 一个通用的列表查询 query
#[derive(Deserialize, TsType)]
struct ListQuery {
    /// 利用类型限制，一次请求的数量不能超过 256 个
    max_count: u8,

    /// 跳过前 offset 个结果
    offset: u32,
}
