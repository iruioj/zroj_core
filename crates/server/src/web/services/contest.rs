use crate::{
    block_it,
    data::contest::{ContestMeta, CtstDB},
    marker::*,
};
use actix_web::web::Json;
use serde::Deserialize;
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};

#[derive(Deserialize, TsType)]
struct CtstMetasQuery {
    #[serde(flatten)]
    list: super::ListQuery,

    /// 搜索的关键字/模式匹配
    pattern: Option<String>,
}

/// 获取比赛列表
#[api(method = get, path = "/metas")]
async fn metas(
    ctst_db: ServerData<CtstDB>,
    query: QueryParam<CtstMetasQuery>,
) -> JsonResult<Vec<ContestMeta>> {
    let query = query.into_inner();
    Ok(Json(block_it!(ctst_db.get_metas(
        query.list.max_count,
        query.list.offset as usize,
        query.pattern,
    ))?))
}

#[scope_service(path = "/contest")]
pub fn service(ctst_db: ServerData<CtstDB>) {
    app_data(ctst_db);
    service(metas);
}
