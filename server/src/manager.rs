pub mod custom_test;
pub mod judge_queue;
pub mod problem;

use crate::auth::{require_login, SessionContainer};
use crate::problem::*;
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_session::Session;
use actix_web::{error, get, post, web, Result};
use judger::{lang::LangOption, TaskResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use judge_queue::JudgeQueue;
use self::{
    custom_test::{start_custom_test, CustomTestManager},
    problem::{ProblemManager, ProblemViewData},
};

type ProblemID = u32;
// type GroupID = i32;

#[derive(Serialize, Debug, Clone, Deserialize)]
pub enum CodeLang {
    #[serde(rename = "gnu_cpp20_o2")]
    GnuCpp20O2,
    #[serde(rename = "gnu_cpp17_o2")]
    GnuCpp17O2,
    #[serde(rename = "gnu_cpp14_o2")]
    GnuCpp14O2,
}

impl LangOption for CodeLang {
    fn build_sigton(&self, source: &PathBuf, dest: &PathBuf) -> sandbox::unix::Singleton {
        match *self {
            Self::GnuCpp14O2 => judger::lang::gnu_cpp14_o2().build_sigton(source, dest),
            Self::GnuCpp17O2 => judger::lang::gnu_cpp17_o2().build_sigton(source, dest),
            Self::GnuCpp20O2 => judger::lang::gnu_cpp20_o2().build_sigton(source, dest),
        }
    }
}
#[get("/{pid}")]
async fn view_problem(
    pid: web::Path<ProblemID>,
    session: Session,
    session_container: web::Data<SessionContainer>,
    manager: web::Data<ProblemManager>,
) -> Result<web::Json<ProblemViewData>> {
    let uid = require_login(&session, &session_container)?;
    if manager.check_access(*pid, uid)? >= ProblemAccess::View {
        Ok(web::Json(manager.fetch_view_data(*pid)?))
    } else {
        Err(error::ErrorBadRequest(
            "You do not have access to this problem",
        ))
    }
}

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
    session: Session,
    session_container: web::Data<SessionContainer>,
    manager: web::Data<CustomTestManager>,
    queue: web::Data<JudgeQueue>,
) -> Result<String> {
    let uid = require_login(&session, &session_container)?;
    let base = manager.get_user_folder(&uid)?;
    let input = base.clone().join("input");
    if let Some(file_name) = payload.source.file_name.clone() {
        let (name, lang) = parse_source_file_name(file_name)?;
        let source = base.clone().join(name);
        std::fs::rename(payload.source.file.path(), &source)
            .map_err(|_| error::ErrorInternalServerError("Fail to move tempfile"))?;
        std::fs::rename(payload.input.file.path(), &input)
            .map_err(|_| error::ErrorInternalServerError("Fail to move tempfile"))?;
        start_custom_test(manager, queue, uid, base, source, input, lang)?;
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
    session: Session,
    session_container: web::Data<SessionContainer>,
    manager: web::Data<CustomTestManager>,
) -> Result<web::Json<CustomTestResult>> {
    let uid = require_login(&session, &session_container)?;
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

/// 提供 problem 的网络服务
pub fn service(
    session_containter: web::Data<SessionContainer>,
    problem_manager: web::Data<ProblemManager>,
    custom_test_manager: web::Data<CustomTestManager>,
    judge_queue: web::Data<JudgeQueue>,
) -> actix_web::Scope {
    web::scope("/api/problem")
        .app_data(session_containter)
        .app_data(problem_manager)
        .app_data(custom_test_manager)
        .app_data(judge_queue)
        .service(view_problem)
        .service(handle_custom_test)
        .service(custom_test_result)
}

/*

use std::sync::{Arc, Mutex};
use crossbeam_channel::{bounded, Receiver, Sender};

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    Job(Job),
    Terminate,
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Message>,
}

struct Worker {
    id: usize,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Self {
        let thread = std::thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::Job(job) => {
                    job();
                }
                Message::Terminate => {
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = bounded::<Message>(size);

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::Job(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
*/
