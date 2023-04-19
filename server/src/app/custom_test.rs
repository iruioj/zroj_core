use crate::{
    auth::{
        middleware::{RequireLogin, RequireSession},
        SessionManager, UserID,
    },
    manager::{
        custom_test::{start_custom_test, CustomTestManager},
        judge_queue::JudgeQueue,
        custom_test::CodeLang,
    },
};
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{error, get, post, web, Result};
use judger::TaskResult;
use serde::{Deserialize, Serialize};

/// warning: this funtion contains probable leak
fn parse_source_file_name(s: String) -> Result<(String, CodeLang)> {
    if s.contains('/') {
        return Err(error::ErrorBadRequest("Invalid source file name"));
    }
    let s = s.trim();
    let split = s.split('.').collect::<Vec<&str>>();
    if split.len() != 3 {
        return Err(error::ErrorBadRequest("Invalid source file name"));
    }
    let lang = split[1];
    let lang = serde_json::from_str(lang).map_err(|_| error::ErrorBadRequest("Unkown language"))?;
    let suffix = split[2];
    Ok(("source.".to_string() + suffix, lang))
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
#[post("/custom_test")]
async fn handle_custom_test(
    payload: MultipartForm<CustomTestPayload>,
    manager: web::Data<CustomTestManager>,
    queue: web::Data<JudgeQueue>,
    uid: web::ReqData<UserID>,
) -> Result<String> {
    let base = manager.get_user_folder(&uid)?;
    let input = base.clone().join("input");
    if let Some(file_name) = payload.source.file_name.clone() {
        let (name, lang) = parse_source_file_name(file_name)?;
        let source = base.clone().join(name);
        std::fs::rename(payload.source.file.path(), &source)
            .map_err(|_| error::ErrorInternalServerError("Fail to move tempfile"))?;
        std::fs::rename(payload.input.file.path(), &input)
            .map_err(|_| error::ErrorInternalServerError("Fail to move tempfile"))?;
        start_custom_test(manager, queue, *uid, base, source, input, lang)?;
        Ok("Judge started".to_string())
    } else {
        Err(error::ErrorBadRequest("Missing source file name"))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomTestResult {
    /// return None if the judging or failed
    pub result: Option<TaskResult>,
}

#[get("/custom_test")]
async fn custom_test_result(
    manager: web::Data<CustomTestManager>,
    uid: web::ReqData<UserID>,
) -> Result<web::Json<CustomTestResult>> {
    Ok(web::Json(CustomTestResult {
        result: manager.fetch_result(&uid)?,
    }))
}
/*
#[get("/{pid}/edit")]
async fn edit(
    pid: web::Path<u32>,
    session: Session,
    session_container: web::Data <SessionContainer>,
    manager: web::Data <ProblemManager>
) -> actix_web::Result <web::Json <ResponseJsonData> > {
    if *pid >= manager.pid_maximum {
        return response_json_data(false, "Problem does not exists", "");
    }
    let uid = fetch_login_state(&session, &session_container)?;
    todo!()
}
*/

/// 提供 manager 的网络服务
pub fn service(
    session_containter: web::Data<SessionManager>,
    custom_test_manager: web::Data<CustomTestManager>,
    judge_queue: web::Data<JudgeQueue>,
) -> actix_web::Scope<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    web::scope("/api")
        .wrap(RequireLogin)
        .wrap(RequireSession)
        .app_data(session_containter)
        .app_data(custom_test_manager)
        .app_data(judge_queue)
        .service(handle_custom_test)
        .service(custom_test_result)
}
