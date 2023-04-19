pub mod custom_test;
pub mod judge_queue;
pub mod problem;

use crate::auth::{UserID};
use crate::problem::*;
use actix_web::{error, get, post, web, Result};
use judger::{lang::LangOption};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use self::{
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
    manager: web::Data<ProblemManager>,
    uid: web::ReqData<UserID>,
) -> Result<web::Json<ProblemViewData>> {
    if manager.check_access(*pid, *uid)? >= ProblemAccess::View {
        Ok(web::Json(manager.fetch_view_data(*pid)?))
    } else {
        Err(error::ErrorBadRequest(
            "You do not have access to this problem",
        ))
    }
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
