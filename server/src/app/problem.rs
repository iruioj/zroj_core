use crate::{
    data::{
        problem_ojdata::OJDataDB,
        problem_statement::{self, StmtDB},
    },
    marker::*,
    // manager::_problem::{Metadata, ProblemManager},
    ProblemID,
};
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{error, web::Json};
use problem::{render_data::statement::StmtMeta, ProblemFullData};
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};
use store::{FsStore, Handle};

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
    if let Some(s) = stmt_db.get(query.id).await? {
        Ok(Json(s))
    } else {
        Err(error::ErrorNotFound("problem not found"))
    }
}

// TODO: 权限/ownership 限制
#[derive(Deserialize, TsType)]
struct MetasQuery {
    // limitations
    /// 利用类型限制，一次请求的数量不能超过 256 个
    max_count: u8,

    // filters
    /// 搜索的关键字/模式匹配
    pattern: Option<String>,
    /// 题目 ID 下限
    min_id: Option<ProblemID>,
    /// 题目 ID 上限
    max_id: Option<ProblemID>,
}

/// 获取题目的元信息
#[api(method = get, path = "/metas")]
async fn metas(
    stmt_db: ServerData<StmtDB>,
    query: QueryParam<MetasQuery>,
) -> JsonResult<Vec<(ProblemID, StmtMeta)>> {
    let query = query.into_inner();
    Ok(Json(
        stmt_db
            .get_metas(query.max_count, query.pattern, query.min_id, query.max_id)
            .await?,
    ))
}

/// 所有的题目元信息，用于调试
#[api(method = get, path = "/full_dbg")]
async fn full_list(stmt_db: ServerData<StmtDB>) -> JsonResult<Vec<(ProblemID, StmtMeta)>> {
    Ok(Json(stmt_db.get_metas(255, None, None, None).await?))
}

#[derive(Debug, MultipartForm)]
struct PostDataPayload {
    /// 题目 id
    /// 如果不指定 id 就新建题目
    id: Option<Text<ProblemID>>,
    /// [`ProblemFullData`] 的压缩文件
    data: TempFile,
}
/// 将文件解压到临时文件夹中
pub fn tempdir_unzip(
    reader: impl std::io::Read + std::io::Seek,
) -> Result<tempfile::TempDir, zip::result::ZipError> {
    let dir = tempfile::TempDir::new()?;
    let mut zip = zip::ZipArchive::new(reader)?;
    zip.extract(dir.path())?;
    Ok(dir)
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
    db: ServerData<OJDataDB>,
    stmt_db: ServerData<StmtDB>,
) -> JsonResult<PostDataReturn> {
    let payload = payload.into_inner();
    let file = payload.data.file.into_file();
    let dir = tempdir_unzip(file).map_err(|e| error::ErrorBadRequest(e))?;
    let id = payload.id.map(|x| x.0).unwrap_or(
        stmt_db
            .max_id()
            .await
            .map_err(|e| error::ErrorInternalServerError(e))?,
    );
    let fulldata =
        ProblemFullData::open(&Handle::new(dir.path())).map_err(|e| error::ErrorBadRequest(e))?;

    db.insert(id, fulldata.data)
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;
    stmt_db
        .insert(id, fulldata.statement)
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    drop(dir); // 限制 dir 的生命周期
    Ok(Json(PostDataReturn { id }))
}

/// 提供 problem 相关服务
#[scope_service(path = "/problem")]
pub fn service(stmt_db: ServerData<StmtDB>, ojdata_db: ServerData<OJDataDB>) {
    app_data(stmt_db);
    app_data(ojdata_db);
    service(full_list);
    service(metas);
    service(statement);
    service(fulldata);
}
