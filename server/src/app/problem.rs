use crate::{
    app::parse_named_file,
    data::{
        problem_ojdata::OJDataDB,
        problem_statement::{self, StmtDB, ProblemMeta},
        submission::SubmDB, types::SubmRaw
    },
    manager::problem_judger::ProblemJudger,
    marker::*,
    ProblemID, SubmID,
};
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{error, web::Json};
use problem::{prelude::*, ProblemFullData};
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

#[derive(Deserialize, TsType)]
struct StmtQuery {
    /// 题目 id
    id: ProblemID,
}

/// 题面数据
#[api(method = get, path = "/statement")]
async fn statement(
    stmt_db: ServerData<StmtDB>,
    query: QueryParam<StmtQuery>,
) -> JsonResult<problem_statement::Statement> {
    Ok(stmt_db.get(query.id).map(Json)?)
}

#[derive(Deserialize, TsType)]
struct ProbMetasQuery {
    #[serde(flatten)]
    list: super::ListQuery,

    /// 搜索的关键字/模式匹配
    pattern: Option<String>,
}

/// 获取题目列表
#[api(method = get, path = "/metas")]
async fn metas(
    stmt_db: ServerData<StmtDB>,
    query: QueryParam<ProbMetasQuery>,
) -> JsonResult<Vec<ProblemMeta>> {
    let query = query.into_inner();
    Ok(Json(stmt_db.get_metas(
        query.list.max_count,
        query.list.offset as usize,
        query.pattern,
    )?))
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

/// 上传题目数据
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

    let id = if let Some(id) = id {
        stmt_db.update(id, fulldata.statement).map(|_| id)
    } else {
        stmt_db.insert_new(fulldata.statement)
    }?;
    ojdata_db.insert(id, fulldata.data).await?;
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
    db.get_meta(query.id)
        .await
        .map_err(error::ErrorInternalServerError)
}

#[derive(Debug, MultipartForm)]
struct JudgePayload {
    pid: Text<ProblemID>,
    files: Vec<TempFile>,
}

#[derive(Debug, Serialize, TsType)]
struct JudgeReturn {
    sid: SubmID,
}

/// 评测题目
#[api(method = post, path = "/submit")]
async fn judge(
    uid: Identity,
    payload: FormData<JudgePayload>,
    judger: ServerData<ProblemJudger>,
    subm_db: ServerData<SubmDB>,
    ojdata_db: ServerData<OJDataDB>,
) -> JsonResult<JudgeReturn> {
    tracing::info!("run judge handler");

    let payload = payload.into_inner();
    let subm_db = subm_db.into_inner();
    let mut raw = SubmRaw(payload.files.iter().filter_map(parse_named_file).collect());

    tracing::info!("request parsed");

    let pid = payload.pid.0;
    let stddata = ojdata_db.get(pid).await?;

    let sid = match stddata {
        problem::StandardProblem::Traditional(ojdata) => {
            let subm_id = subm_db.insert_new(
                *uid,
                pid,
                raw.0.get("source").map(|x| x.file_type.clone()),
                // must insert_new before consuming raw
                &raw,
            )?;

            let subm = TraditionalSubm {
                source: raw
                    .0
                    .remove("source")
                    .ok_or(error::ErrorBadRequest("source file not found"))?,
            };
            judger.add_test::<_, _, _, Traditional>(subm_id, ojdata, subm)?;
            subm_id
        }
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
}
