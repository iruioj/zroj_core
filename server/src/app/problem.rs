use crate::{
    data::problem_statement::{self, StmtDB},
    marker::*,
    // manager::_problem::{Metadata, ProblemManager},
    ProblemID,
};
use actix_web::{error, web::Json};
use problem::render_data::statement::Meta;
use serde::Deserialize;
use serde_ts_typing::SerdeJsonWithType;
use server_derive::{api, scope_service};

#[derive(Deserialize, SerdeJsonWithType)]
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

/// 所有的题目元信息，用于调试
#[api(method = get, path = "/full_dbg")]
async fn full_list(stmt_db: ServerData<StmtDB>) -> JsonResult<Vec<(ProblemID, Meta)>> {
    Ok(Json(stmt_db.get_metas().await?))
}

/// 提供 problem 相关服务
#[scope_service(path = "/problem")]
pub fn service(stmt_db: ServerData<StmtDB>) {
    app_data(stmt_db);
    service(full_list);
    service(statement);
}
