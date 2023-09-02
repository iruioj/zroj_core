use actix_web::{error::ErrorInternalServerError, web::Json};
use judger::truncstr::TruncStr;
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};

use crate::{
    data::submission::{FilterOption, SubmDB, SubmInfo, SubmMeta},
    manager::problem_judger::ProblemJudger,
    marker::*,
    SubmID,
};

use super::ListQuery;

#[derive(TsType, Serialize)]
struct DetailReturn {
    info: SubmInfo,
    /// 提交记录的源文件内容
    raw: Vec<(String, judger::FileType, TruncStr)>,
    /// 如果正在评测，就返回评测日志
    judge: Option<Vec<String>>,
}

#[derive(TsType, Deserialize)]
struct DetailQuery {
    sid: SubmID,
}

const SOURCE_LIMIT: usize = 100 * 1024;

/// 查询提交记录
#[api(method = get, path = "/detail")]
async fn detail(
    payload: QueryParam<DetailQuery>,
    subm_db: ServerData<SubmDB>,
    judger: ServerData<ProblemJudger>,
) -> JsonResult<DetailReturn> {
    let logs = judger.get_logs(&payload.sid)?;
    if let Some(logs) = &logs {
        // if judge finished, then store the result
        if logs.iter().any(|o| matches!(o, judger::LogMessage::Done)) {
            let (report, _) = judger
                .remove_result(&payload.sid)
                .ok_or(ErrorInternalServerError("judge result not found"))?;
            subm_db.update(&payload.sid, report).await?;
        }
    }

    let meta = subm_db.get(&payload.sid).await?;
    let raw = subm_db
        .get_raw(&payload.sid)
        .await?
        .0
        .into_iter()
        .filter(|(_, v)| !matches!(v.file_type, judger::FileType::Binary))
        .map(|(k, mut v)| {
            Ok((
                k,
                v.file_type.clone(),
                TruncStr::new(v.read_to_string()?, SOURCE_LIMIT),
            ))
        })
        .collect::<Result<Vec<_>, std::io::Error>>()
        .map_err(|e| ErrorInternalServerError(e))?;

    Ok(Json(DetailReturn {
        info: meta.into(),
        raw,
        judge: logs.map(|v| v.into_iter().map(|s| s.to_string()).collect()),
    }))
}

#[derive(Deserialize, TsType)]
struct MetasQuery {
    list: ListQuery,

    filter: FilterOption,
}

/// 获取提交记录列表
#[api(method = get, path = "/metas")]
async fn metas(
    subm_db: ServerData<SubmDB>,
    query: QueryParam<MetasQuery>,
) -> JsonResult<Vec<SubmMeta>> {
    let query = query.into_inner();
    Ok(Json(
        subm_db
            .get_metas(query.list.max_count, query.list.offset as usize, query.filter)
            .await?,
    ))
}

/// 提供 problem 相关服务
#[scope_service(path = "/submission")]
pub fn service(subm_db: ServerData<SubmDB>, judger: ServerData<ProblemJudger>) {
    app_data(subm_db);
    app_data(judger);
    service(detail);
    service(metas);
}
