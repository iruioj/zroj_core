use crate::{
    data::problemdata::{self, StmtDB},
    marker::*,
    // manager::_problem::{Metadata, ProblemManager},
    ProblemID,
};
use actix_web::{error, web::Json};
use problem::render_data::statement::StmtMeta;
use serde::Deserialize;
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};

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
) -> JsonResult<problemdata::Statement> {
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
async fn metas(stmt_db: ServerData<StmtDB>, query: QueryParam<MetasQuery>) -> JsonResult<Vec<(ProblemID, StmtMeta)>> {
    let query = query.into_inner();
    Ok(Json(stmt_db.get_metas(query.max_count, query.pattern, query.min_id, query.max_id).await?))
}

/// 所有的题目元信息，用于调试
#[api(method = get, path = "/full_dbg")]
async fn full_list(stmt_db: ServerData<StmtDB>) -> JsonResult<Vec<(ProblemID, StmtMeta)>> {
    Ok(Json(stmt_db.get_metas(255, None, None, None).await?))
}

/// 提供 problem 相关服务
#[scope_service(path = "/problem")]
pub fn service(stmt_db: ServerData<StmtDB>) {
    app_data(stmt_db);
    service(full_list);
    service(metas);
    service(statement);
}
