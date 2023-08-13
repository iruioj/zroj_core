use crate::{manager::one_off::OneOffManager, marker::*, UserID};
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{
    error::ErrorBadRequest,
    web,
};
use judger::{StoreFile, TaskReport};
use serde::Serialize;
use serde_json::json;
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};
use std::fmt::Debug;

/// warning: this funtion contains probable leak
fn parse_source_name(s: String) -> Option<judger::FileType> {
    let lang = s.trim().split('.').nth(1).unwrap();
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
    payload: FormData<CustomTestPayload>,
    oneoff: ServerData<OneOffManager>,
    uid: web::ReqData<UserID>,
) -> actix_web::Result<String> {
    let Some(file_name) = payload.source.file_name.clone() else {
        return Err(ErrorBadRequest("missing source file name"))
    };
    let lang = parse_source_name(file_name).ok_or(ErrorBadRequest("invalid file name"))?;
    if !lang.compileable() {
        return Err(ErrorBadRequest("file not compilable"))
    }
    let source = StoreFile {
        file: payload.source.file.reopen()?,
        file_type: lang,
    };
    let input = StoreFile {
        file: payload.input.file.reopen()?,
        file_type: judger::FileType::Plain,
    };
    oneoff.add_test(*uid, source, input)?;
    Ok("Judge started".to_string())
}

#[derive(Debug, Serialize, TsType)]
pub struct CustomTestResult {
    /// return None if the judging or failed
    pub result: Option<TaskReport>,
}

#[api(method = get, path = "")]
async fn custom_test_get(
    oneoff: web::Data<OneOffManager>,
    uid: web::ReqData<UserID>,
) -> JsonResult<CustomTestResult> {
    Ok(web::Json(CustomTestResult {
        result: oneoff.get_result(&uid)?,
    }))
}

/// 提供自定义测试服务
#[scope_service(path = "/custom_test")]
pub fn service(custom_test_manager: web::Data<OneOffManager>) {
    app_data(custom_test_manager);
    service(custom_test_get);
    service(custom_test_post);
}
