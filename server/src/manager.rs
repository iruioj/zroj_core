use std::path::PathBuf;
use std::sync::{RwLock, Arc};
use actix_multipart::form::MultipartForm;
use actix_web::{
    get, post, web,
    Result, error,
};
use actix_session::{Session};
use judger::lang::LangOption;
use serde::{Serialize, Deserialize};
use crate::schema::CustomTestResult;
use crate::{
    auth::{
        SessionContainer,
        UserID,
        require_login,
    },
    config::ServerConfig,
    schema::{ CustomTestPayload, }
};
use crate::problem::*;
type ProblemID = u32;
type GroupID = i32;
use judger::{OneOff, JudgeResult};

/// For page /problem/{pid}, api url /api/problem/{pid}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProblemViewData {
    general_config:  GeneralConfig,
    statement: StatementViewData,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
enum StatementViewData {
    /// given source code and do client side render
    Markdown(StatementSource),
    /// previously rendered tex into html
    LaTex(String)
}


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

pub mod judge_queue {
    use std::sync::{Arc, Mutex};
    use crossbeam_channel::{bounded, Receiver, Sender};
    type Job = Box<dyn FnOnce() + Send + 'static>;
    enum Message {
        Job(Job),
        Terminate,
    }
    struct Worker {
        thread: Option<std::thread::JoinHandle<()>>,
    }
    impl Worker {
        fn new(id: usize, receiver: Arc <Mutex<Receiver<Message>>>) -> Self {
            let thread = std::thread::spawn(move || loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::Job(job) => {
                        println!("Judge work {} is judging", id);
                        job();
                    }
                    Message::Terminate => {
                        break;
                    }
                }
            });
            Worker {
                thread: Some(thread),
            }
        }
    }
    pub struct JudgeQueue {
        workers: Vec<Worker>,
        sender: Sender<Message>,
    }
    impl JudgeQueue {
        pub fn new(size: usize) -> Self {
            assert!(size > 0);
            let (sender, receiver) = bounded::<Message>(size);
            let receiver = Arc::new(Mutex::new(receiver));
            let mut workers = Vec::with_capacity(size);
            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }
            JudgeQueue { workers, sender }
        }
        pub fn add <F> (&self, f: F)
        where
            F: FnOnce() + Send + 'static,
        {
            let job = Box::new(f);
            self.sender.send(Message::Job(job)).unwrap();
        }
    }
    impl Drop for JudgeQueue {
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
}
use judge_queue::JudgeQueue;

#[derive(Debug)]
pub struct ProblemManager {
    locks: Vec <RwLock<()> >,
    /// base directory of each problem
    base_dir: String,
    /// the json file that store problem statement
    statement: String,
    /// the directory that stores problem data
    data_dir: String,
    pid_maximum: ProblemID,
}
impl ProblemManager {
    pub fn new(config: &ServerConfig) -> Self {
        Self {
            locks: (0..config.pid_maximum).map(|_| RwLock::new(())).collect(),
            base_dir: config.problem_base_dir.clone(),
            statement: config.problem_statement.clone(),
            data_dir: config.problem_data_dir.clone(),
            pid_maximum: config.pid_maximum.clone(),
        }
    }
    fn fetch_file(&self, path: &String) -> Result <String> {
        std::fs::read_to_string(path).
            map_err(|e| error::ErrorInternalServerError(e.to_string()))
    }
    fn get_base_dir(&self, pid: ProblemID) -> Result <String> {
        let mut s = self.base_dir.clone();
        if let None = s.find("{}") {
            return Err(error::ErrorInternalServerError("Problem base dir is not correct. {} is required".to_string()));
        }
        s = s.replace("{}", &pid.to_string());
        if let Some(_) = s.find("{}") {
            return Err(error::ErrorInternalServerError("Problem base dir is not correct. Too many {}s".to_string()));
        }
        Ok(s)
    }
    fn read_statement(&self, pid: ProblemID) -> Result <String> {
        let guard = self.locks[pid as usize]
            .read()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        let dir = self.get_base_dir(pid)? + &self.statement;
        let result = self.fetch_file(&dir)?;
        drop(guard);
        Ok(result)
    }
    fn check_access(&self, pid: ProblemID, uid: UserID) -> actix_web::Result <ProblemAccess> {
        todo!()
    }
    fn fetch_view_data(&self, pid: ProblemID) -> actix_web::Result <ProblemViewData> {
        todo!()
    }
}

#[get("/{pid}")]
async fn view_problem(
    pid: web::Path<ProblemID>,
    session: Session,
    session_container: web::Data <SessionContainer>,
    manager: web::Data <ProblemManager>
) -> Result <web::Json <ProblemViewData> > {
    if *pid >= manager.pid_maximum {
        return Err(error::ErrorBadRequest("Problem id too large"));
    }
    let uid = require_login(&session, &session_container)?;
    if manager.check_access(*pid, uid)? >= ProblemAccess::View {
        Ok(web::Json(manager.fetch_view_data(*pid)?))
    } else {
        Err(error::ErrorBadRequest("You do not have access to this problem"))
    }
}

/*
        let mut one = OneOff::new(src.into(), inp.into(), gnu_cpp17_o2());
        one.set_wd(dir.path().to_path_buf());
        let res = one.exec()?;
*/

#[derive(Debug)]
pub struct CustomTestManager {
    /// base directory of each problem
    base_dir: String,
    uid_maximum: UserID,
    state: Arc<Vec<RwLock <Option <JudgeResult>>>>,
}
impl CustomTestManager {
    pub fn new(config: &ServerConfig) -> Self {
        Self {
            base_dir: config.problem_base_dir.clone(),
            uid_maximum: config.uid_maximum.clone(),
            state: Arc::new((0..config.uid_maximum).map(|_| RwLock::new(None)).collect()),
        }
    }
    pub fn check_userid(&self, uid: &UserID) -> Result <()> {
        if *uid < 0 || *uid > self.uid_maximum {
            return Err(error::ErrorInternalServerError("User id too large"));
        }
        Ok(())
    }
    pub fn fetch_result(&self, uid: &UserID) -> Result <Option <JudgeResult> > {
        self.check_userid(uid)?;
        let guard = self.state[*uid as usize].
            read().
            map_err(|_| error::ErrorInternalServerError("Fail to get lock"))?;
        Ok((*guard).clone())
    }
    fn get_user_folder(&self, uid: &UserID) -> Result <PathBuf> {
        let mut path = PathBuf::new();
        path.push(&self.base_dir);
        path = path.join(uid.to_string());
        let b = path.is_dir();
        if !b {
            std::fs::create_dir(&path)
                .map_err(
                    |_| {error::ErrorInternalServerError(
                        format!("Fail to setup user custom test directory: {}", path.to_string_lossy())
                    )}
                )?;
        }
        Ok(path)
    }
}


/// warning: this funtion contains probable leak
fn parse_source_file_name(mut s: String) -> Result <(String, CodeLang)> {
    if s.contains('/') {
        return Err(error::ErrorBadRequest("Invalid source file name"));
    }
    let s = s.trim();
    let split = s.split('.').collect :: <Vec <&str>>();
    if split.len() != 3 {
        return Err(error::ErrorBadRequest("Invalid source file name"));
    }
    let lang = split[1];
    let lang = serde_json::from_str(lang)
        .map_err(|_| {
            error::ErrorBadRequest("Unkown language")
        })?;
    let suffix = split[2];
    Ok(("source.".to_string() + suffix, lang))
}
fn start_custom_test(
    manager: web::Data <CustomTestManager>,
    queue: web::Data <JudgeQueue>,
    uid: UserID,
    base: PathBuf,
    source: PathBuf,
    input: PathBuf,
    lang: CodeLang
) -> Result <()> {
    manager.check_userid(&uid)?;
    let state = manager.state.clone();
    queue.add(move || {
        let mut one = OneOff::new(
            source,
            Some(input),
            lang
        );
        one.set_wd(base);
        let result = one.exec();
        if let Ok(mut guard) = state[uid as usize].write() {
            *guard = match(result) {
                Ok(result) => Some(result),
                Err(_) => None,
            }
        } else {
            eprintln!("Fail to write judge result, cannot retrive lock");
        }
    });
    Ok(())
}

#[post("/custom_test")]
async fn custom_test(
    payload: MultipartForm <CustomTestPayload>,
    session: Session,
    session_container: web::Data <SessionContainer>,
    manager: web::Data <CustomTestManager>,
    queue: web::Data <JudgeQueue>,
) -> Result <String> {
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

#[get("/custom_test")]
async fn custom_test_result(
    session: Session,
    session_container: web::Data <SessionContainer>,
    manager: web::Data <CustomTestManager>,
) -> Result <web::Json <CustomTestResult> > {
    let uid = require_login(&session, &session_container)?;
    Ok(web::Json(CustomTestResult{result: manager.fetch_result(&uid)?}))
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
    session_containter: web::Data <SessionContainer>,
    problem_manager: web::Data <ProblemManager>,
    custom_test_manager: web::Data <CustomTestManager>,
    judge_queue: web::Data <JudgeQueue>
) -> actix_web::Scope {
    web::scope("/api/problem")
        .app_data(session_containter)
        .app_data(problem_manager)
        .app_data(custom_test_manager)
        .app_data(judge_queue)
        .service(view_problem)
        .service(custom_test)
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