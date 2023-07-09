use crate::{
    data::problem_statement::StmtDB,
    marker::*,
    // manager::_problem::{Metadata, ProblemManager},
    ProblemID,
};
use actix_web::web::Json;
use problem::render_data::statement::Meta;
use server_derive::{api, scope_service};

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
}
