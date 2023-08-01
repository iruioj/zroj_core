use crate::{
    manager::{
        custom_test::{start_custom_test, CustomTestManager},
        judge_queue::JudgeQueue,
    },
    UserID, marker::JsonResult,
};
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{error::ErrorBadRequest, web};
use judger::{StoreFile, TaskReport};
use serde::Serialize;
use serde_json::json;
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};
use std::fmt::Debug;

/// warning: this funtion contains probable leak
fn parse_source_name(s: String) -> Option<judger::FileType> {
    let lang = s.trim().split('.').skip(1).next().unwrap();
    serde_json::from_value(json!(lang)).ok()
}

/// format of custom test post payload
#[derive(Debug, MultipartForm)]
pub struct CustomTestPayload {
    #[multipart]
    /// source file, file name: any.{lang}.{suf}
    pub source: TempFile,
    /// input file
    #[multipart]
    pub input: TempFile,
}

#[api(method = post, path = "")]
async fn custom_test_post(
    payload: MultipartForm<CustomTestPayload>,
    manager: web::Data<CustomTestManager>,
    queue: web::Data<JudgeQueue>,
    uid: web::ReqData<UserID>,
) -> actix_web::Result<String> {
    if let Some(file_name) = payload.source.file_name.clone() {
        let lang = parse_source_name(file_name).expect("invalid file name");
        let source = StoreFile {
            file: payload.source.file.reopen()?,
            file_type: lang,
        };
        let input = StoreFile {
            file: payload.input.file.reopen()?,
            file_type: judger::FileType::Plain,
        };
        start_custom_test(manager, queue, *uid, source, input)?;
        Ok("Judge started".to_string())
    } else {
        Err(ErrorBadRequest(format!(
            "missing source file name, source size = {}, input size = {}",
            payload.source.size, payload.input.size
        )))
    }
}

#[derive(Debug, Serialize, TsType)]
pub struct CustomTestResult {
    /// return None if the judging or failed
    pub result: Option<TaskReport>,
}

#[api(method = get, path = "")]
async fn custom_test_get(
    manager: web::Data<CustomTestManager>,
    uid: web::ReqData<UserID>,
) -> JsonResult<CustomTestResult> {
    Ok(web::Json(CustomTestResult {
        result: manager.fetch_result(&uid)?,
    }))
}

/// 提供自定义测试服务
#[scope_service(path = "/custom_test")]
pub fn service(
    custom_test_manager: web::Data<CustomTestManager>,
    judge_queue: web::Data<JudgeQueue>,
) {
    app_data(custom_test_manager);
    app_data(judge_queue);
    service(custom_test_get);
    service(custom_test_post);
}
