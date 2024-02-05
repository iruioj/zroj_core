use crate::{
    app::parse_named_file, auth::Authentication, manager::one_off::OneOffManager, marker::*,
};
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{
    error::{ErrorBadRequest, ErrorUnauthorized},
    web::Json,
};
use judger::{StoreFile, TaskReport};
use serde::Serialize;
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};
use std::fmt::Debug;

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
    auth: Authentication,
) -> AnyResult<String> {
    let Some(uid) = auth.user_id() else {
        return Err(ErrorUnauthorized("no user info"));
    };
    let Some((_, source)) = parse_named_file(&payload.source) else {
        return Err(ErrorBadRequest("invalid payload file"));
    };
    if !source.file_type.compileable() {
        return Err(ErrorBadRequest("file not compilable"));
    }
    let input = StoreFile {
        file: payload.input.file.reopen()?,
        file_type: judger::FileType::Plain,
    };
    oneoff.add_test(uid, source, input)?;
    Ok("Judge started".to_string())
}

#[derive(Debug, Serialize, TsType)]
pub struct CustomTestResult {
    /// return None if the judging or failed
    pub result: Option<TaskReport>,
}

#[api(method = get, path = "")]
async fn custom_test_get(
    oneoff: ServerData<OneOffManager>,
    auth: Authentication,
) -> JsonResult<CustomTestResult> {
    let Some(uid) = auth.user_id() else {
        return Err(ErrorUnauthorized("no user info"));
    };
    Ok(Json(CustomTestResult {
        result: oneoff.get_result(&uid)?,
    }))
}

/// 提供自定义测试服务
#[scope_service(path = "/custom_test")]
pub fn service(custom_test_manager: ServerData<OneOffManager>) {
    app_data(custom_test_manager);
    service(custom_test_get);
    service(custom_test_post);
}
