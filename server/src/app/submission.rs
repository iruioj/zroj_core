use actix_web::web::Json;
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};

use crate::{
    data::submission::{SubmDB, SubmInfo, SubmMeta},
    manager::problem_judger::ProblemJudger,
    marker::*,
    SubmID,
};

#[derive(TsType, Serialize)]
struct DetailReturn {
    info: SubmInfo,
    /// 如果正在评测，就返回评测日志
    judge: Option<Vec<String>>,
}

#[derive(TsType, Deserialize)]
struct DetailQuery {
    sid: SubmID,
}

/// 查询提交记录
#[api(method = get, path = "/detail")]
async fn detail(
    payload: QueryParam<DetailQuery>,
    subm_db: ServerData<SubmDB>,
    judger: ServerData<ProblemJudger>,
) -> JsonResult<DetailReturn> {
    let logs = judger.get_logs(&payload.sid)?;
    let info = subm_db.get_info(&payload.sid)?;
    // let raw = subm_db.get_raw(&payload.sid)?.to_display_vec();

    Ok(Json(DetailReturn {
        info,
        judge: logs.map(|v| v.into_iter().map(|s| s.to_string()).collect()),
    }))
}

#[derive(Deserialize, TsType)]
struct SubmMetasQuery {
    #[serde(flatten)]
    list: super::ListQuery,

    pid: Option<crate::ProblemID>,
    uid: Option<crate::UserID>,
    lang: Option<judger::FileType>,
}

/// 获取提交记录列表
#[api(method = get, path = "/metas")]
async fn metas(
    subm_db: ServerData<SubmDB>,
    query: QueryParam<SubmMetasQuery>,
) -> JsonResult<Vec<SubmMeta>> {
    let query = query.into_inner();
    Ok(Json(subm_db.get_metas(
        query.list.max_count,
        query.list.offset as usize,
        query.pid,
        query.uid,
        query.lang,
    )?))
}

/// 提供 problem 相关服务
#[scope_service(path = "/submission")]
pub fn service(subm_db: ServerData<SubmDB>, judger: ServerData<ProblemJudger>) {
    app_data(subm_db);
    app_data(judger);
    service(detail);
    service(metas);
}