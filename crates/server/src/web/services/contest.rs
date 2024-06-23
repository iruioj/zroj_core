use crate::{
    block_it,
    data::{
        databases::contest::{ContestInfo, ContestMeta, CtstDB, UserMeta},
        PermissionManager,
    },
    marker::*,
    web::auth::Authentication,
    CtstID,
};
use actix_web::web::Json;
use serde::Deserialize;
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};

#[derive(Deserialize, TsType)]
struct CtstMetasQuery {
    #[serde(flatten)]
    list: super::ListQuery,

    /// pattern matching
    pattern: Option<String>,
}

/// Fetch the list of contests
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

#[derive(Deserialize, TsType)]
struct CtstQuery {
    /// Contest id
    id: CtstID,
}

/// 获取比赛主页信息
#[api(method = get, path = "/info")]
async fn info(
    ctst_db: ServerData<CtstDB>,
    query: QueryParam<CtstQuery>,
    perm_manager: ServerData<PermissionManager>,
    auth: Authentication,
) -> JsonResult<ContestInfo> {
    let user_id = auth.user_id_or_unauthorized()?;
    let cid = query.id;

    let info = block_it!({
        let rs = ctst_db.rs_get(cid);
        perm_manager.query(user_id, rs)
    })?;
    Ok(Json(info))
}

#[derive(Deserialize, TsType)]
struct CtstRegistQuery {
    #[serde(flatten)]
    list: super::ListQuery,

    /// 比赛 id
    id: CtstID,
}

/// 获取比赛报名用户列表
#[api(method = get, path = "/registrants")]
async fn registrants(
    ctst_db: ServerData<CtstDB>,
    query: QueryParam<CtstRegistQuery>,
    perm_manager: ServerData<PermissionManager>,
    auth: Authentication,
) -> JsonResult<Vec<UserMeta>> {
    let user_id = auth.user_id_or_unauthorized()?;
    let r = block_it!({
        let rs =
            ctst_db.rs_get_registrants(query.id, query.list.max_count, query.list.offset as usize);
        perm_manager.query(user_id, rs)
    })?;
    Ok(Json(r))
}

#[derive(Deserialize, TsType)]
struct CtstRegistInfo {
    cid: CtstID,
}

/// 添加比赛报名用户
#[api(method = post, path = "/registrants")]
async fn registrant_post(
    ctst_db: ServerData<CtstDB>,
    reg_info: JsonBody<CtstRegistInfo>,
    auth: Authentication,
    perm_manager: ServerData<PermissionManager>,
) -> AnyResult<String> {
    let uid = auth.user_id_or_unauthorized()?;
    block_it!({
        let rs = ctst_db.rs_insert_registrant(reg_info.cid, uid);
        perm_manager.query(uid, rs)
    })?;
    Ok("ok".into())
}

/// 删除比赛报名用户
#[api(method = delete, path = "/registrants")]
async fn registrant_delete(
    ctst_db: ServerData<CtstDB>,
    reg_info: JsonBody<CtstRegistInfo>,
    auth: Authentication,
    perm_manager: ServerData<PermissionManager>,
) -> AnyResult<String> {
    let uid = auth.user_id_or_unauthorized()?;
    block_it!({
        let rs = ctst_db.rs_remove_registrant(reg_info.cid, uid);
        perm_manager.query(uid, rs)
    })?;
    Ok("ok".into())
}

#[scope_service(path = "/contest")]
pub fn service(ctst_db: ServerData<CtstDB>) {
    app_data(ctst_db);
    service(metas);
    service(info);
    service(registrants);
    service(registrant_post);
    service(registrant_delete);
}
