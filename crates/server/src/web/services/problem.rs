use std::os::unix::fs::MetadataExt;

use crate::{
    block_it,
    data::{
        problem_ojdata::OJDataDB,
        problem_statement::{self, ProblemMeta},
        submission::{SubmDB, SubmInfo},
        types::SubmRaw,
    },
    manager::ProblemJudger,
    marker::*,
    web::{auth::Authentication, services::parse_named_file},
    CtstID, ProblemID, SubmID,
};
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{error, http::header::ContentDisposition, web::Json};
use problem::{prelude::*, render_data::Mdast, ProblemFullData};
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};
use store::{FsStore, Handle};

/// 将文件解压到临时文件夹中
fn tempdir_unzip(
    reader: impl std::io::Read + std::io::Seek,
) -> Result<tempfile::TempDir, zip::result::ZipError> {
    let dir = tempfile::TempDir::new()?;
    let mut zip = zip::ZipArchive::new(reader)?;
    zip.extract(dir.path())?;
    Ok(dir)
}

type StmtDB = problem_statement::StmtDB;

#[derive(Deserialize, TsType)]
struct StmtQuery {
    /// 题目 id
    id: ProblemID,
}

/// Get the problem statement. Currently only one default statement is returned.
/// It will be extended to support i18n.
#[api(method = get, path = "/statement")]
async fn statement(
    stmt_db: ServerData<StmtDB>,
    query: QueryParam<StmtQuery>,
) -> JsonResult<problem_statement::Statement> {
    let pid = query.id;
    let mut ast = block_it!(stmt_db.get(pid))?;
    #[inline]
    fn is_extern_link(link: &str) -> bool {
        link.contains("://")
    }
    fn replace_assets_link(node: &mut Mdast, pid: u32) {
        if let Mdast::Link(link) = node {
            if !is_extern_link(&link.url) {
                link.url = format!("/api/problem/statement_assets?id={pid}&name={}", link.url)
            }
        }
        for c in node.child_nodes().unwrap_or_default() {
            replace_assets_link(c, pid)
        }
    }
    replace_assets_link(&mut ast.statement, pid);
    Ok(Json(ast))
}

#[derive(Deserialize, TsType)]
struct StmtAssetQuery {
    /// 题目 id
    id: ProblemID,
    /// 文件相对路径
    name: String,
}

pub const PDF_INLINE_SIZE: u64 = 5 * 1024 * 1024;

/// 获取某个题目的附加文件，如果不存在就去获取全局的附加文件
#[api(method = get, path = "/statement_assets")]
async fn statement_assets(
    stmt_db: ServerData<StmtDB>,
    query: QueryParam<StmtAssetQuery>,
) -> AnyResult<actix_files::NamedFile> {
    let mut ret = block_it!(stmt_db.get_assets(query.id, &query.name))?;
    // make small pdf files inline for frontend problem display
    if ret.path().extension().is_some_and(|e| e == "pdf")
        && ret
            .file()
            .metadata()
            .inspect_err(|e| tracing::warn!(error = ?e, "get metadata error"))
            .is_ok_and(|m| m.size() <= PDF_INLINE_SIZE)
    {
        ret = ret.set_content_disposition(ContentDisposition {
            disposition: actix_web::http::header::DispositionType::Inline,
            parameters: Vec::new(),
        });
        tracing::info!("make asset inline: {:?}", ret.path());
    }
    Ok(ret)
}

#[derive(Deserialize, TsType)]
struct ProbMetasQuery {
    #[serde(flatten)]
    list: super::ListQuery,

    /// 搜索的关键字/模式匹配
    pattern: Option<String>,
}

/// 获取题目列表。
/// 后端的 `max_count` 为 u8 类型，限制了此 API 返回的题目数最多为 255 个
#[api(method = get, path = "/metas")]
async fn metas(
    stmt_db: ServerData<StmtDB>,
    query: QueryParam<ProbMetasQuery>,
) -> JsonResult<Vec<ProblemMeta>> {
    let query = query.into_inner();
    Ok(Json(block_it!(stmt_db.get_metas(
        query.list.max_count,
        query.list.offset as usize,
        query.pattern,
    ))?))
}

#[derive(Debug, MultipartForm)]
struct PostDataPayload {
    /// 题目 id
    /// 如果不指定 id 就新建题目
    id: Option<Text<ProblemID>>,

    /// [`ProblemFullData`] 的压缩文件
    data: TempFile,
}

#[derive(Debug, Serialize, TsType)]
struct PostDataReturn {
    // 返回题目的 id
    id: ProblemID,
}

/// Upload the problem data. The HTTP request body is a formdata composed of
///
/// - An optional text field `id` (if not found, a new problem will be created)
/// - A binary file `data` containing the content of a zip file. This file is often
///   created by problem configuring tools, which can be safely opened as [`ProblemFullData`].
///
#[api(method = post, path = "/fulldata")]
async fn fulldata(
    payload: FormData<PostDataPayload>,
    ojdata_db: ServerData<OJDataDB>,
    stmt_db: ServerData<StmtDB>,
) -> JsonResult<PostDataReturn> {
    let payload = payload.into_inner();
    let file = payload.data.file.into_file();
    let dir = tempdir_unzip(file).map_err(error::ErrorBadRequest)?;
    let id = payload.id.map(|x| x.0);
    let fulldata =
        ProblemFullData::open(&Handle::new(dir.path())).map_err(error::ErrorBadRequest)?;

    let id = block_it!(if let Some(id) = id {
        stmt_db.update(id, fulldata.statement).map(|_| id)
    } else {
        stmt_db.insert_new(fulldata.statement)
    })?;
    ojdata_db.insert(id, fulldata.data)?;
    // stmt_db.insert(id, fulldata.statement)?;

    drop(dir); // 限制 dir 的生命周期
    Ok(Json(PostDataReturn { id }))
}

#[derive(Debug, TsType, Deserialize)]
struct FullDataMetaQuery {
    id: ProblemID,
}
/// 题目数据元信息
#[api(method = get, path = "/fulldata_meta")]
async fn fulldata_meta(
    query: QueryParam<FullDataMetaQuery>,
    db: ServerData<OJDataDB>,
) -> AnyResult<String> {
    let prob = db.get(query.id)?;
    Ok(format!("{:#?}", prob))
}

#[derive(Debug, MultipartForm)]
struct JudgePayload {
    pid: Option<Text<ProblemID>>,
    cid: Option<Text<CtstID>>,
    sid: Option<Text<SubmID>>,
    files: Vec<TempFile>,
}

#[derive(Debug, Serialize, TsType)]
struct JudgeReturn {
    sid: SubmID,
}

/// Problem judge. User's submission can be seen as a series of files each named
/// `name.lang.ext`. The HTTP request body is composed of a form data, containing
/// a text field `pid` and a list of named files, which is coverted to [`SubmRaw`].
/// Here's an example of frontend payload construction:
///
/// ```javascript
/// const form = new FormData();
///
/// /// Case 1: post a new submission
/// form.append("pid", problem_id.to_string());
/// form.append("cid", contest_id.to_string()); // this is optional
///
/// // append will not override existing key-value pair
/// form.append(
///   "files",
///   new File([s.payload], `source.${lang.value!.value}.cpp`),
/// );
///
/// /// Case 2: post a rejudge submission
/// form.append("sid", submission_id.to_string());
/// ```
///
/// See [`parse_named_file`] for more information.
///
/// Different problems require different submission format. It is encouraged to
/// implement UI for each of the buildin problems (e.g. stdio problem, interactive
/// problem, etc.), and a general UI for any custom problem.
///
#[api(method = post, path = "/submit")]
async fn judge(
    auth: Authentication,
    payload: FormData<JudgePayload>,
    judger: ServerData<ProblemJudger>,
    subm_db: ServerData<SubmDB>,
    ojdata_db: ServerData<OJDataDB>,
) -> JsonResult<JudgeReturn> {
    let uid = auth.user_id_or_unauthorized()?;
    let payload = payload.into_inner();
    let subm_db = subm_db.into_inner();
    let mut raw = SubmRaw(payload.files.iter().filter_map(parse_named_file).collect());

    let sid = if let Some(Text(pid)) = payload.pid {
        let stddata = ojdata_db.get(pid)?;
        let cid = payload.cid.as_ref().map(|o| o.0);

        match stddata {
            problem::StandardProblem::Traditional(ojdata) => {
                let raw2 = raw.clone();
                let subm_id = block_it! {
                    let file_type = raw2.get("source").map(|x| x.file_type.clone());
                    subm_db.insert_new(uid, pid, cid, file_type, &raw2)
                }?;

                let subm = traditional::Subm {
                    source: raw
                        .remove("source")
                        .ok_or(error::ErrorBadRequest("source file not found"))?,
                };
                judger
                    .add_test::<Traditional>(subm_id, ojdata, subm)
                    .map_err(error::ErrorInternalServerError)?;
                subm_id
            }
            _ => todo!(),
        }
    } else {
        let Some(Text(sid)) = payload.sid else {
            return Err(error::ErrorBadRequest("pid or sid not found"));
        };
        tracing::info!(?sid, "rejudge problem");

        let SubmInfo { mut raw, meta, .. } = subm_db.get_info(&sid)?;
        let stddata = ojdata_db.get(meta.pid)?;

        match stddata {
            problem::StandardProblem::Traditional(ojdata) => {
                let subm = traditional::Subm {
                    source: raw
                        .remove("source")
                        .ok_or(error::ErrorBadRequest("source file not found"))?,
                };
                judger
                    .add_test::<Traditional>(sid, ojdata, subm)
                    .map_err(error::ErrorInternalServerError)?;
            }
            _ => todo!(),
        }

        sid
    };

    Ok(Json(JudgeReturn { sid }))
}

/// 提供 problem 相关服务
#[scope_service(path = "/problem")]
pub fn service(
    stmt_db: ServerData<StmtDB>,
    ojdata_db: ServerData<OJDataDB>,
    subm_db: ServerData<SubmDB>,
    judger: ServerData<ProblemJudger>,
) {
    app_data(stmt_db);
    app_data(ojdata_db);
    app_data(subm_db);
    app_data(judger);
    service(metas);
    service(statement);
    service(fulldata);
    service(fulldata_meta);
    service(judge);
    service(statement_assets);
}
