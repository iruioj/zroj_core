use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::{Arc, Mutex};
type Job = Box<dyn FnOnce() + Send + 'static>;
enum Message {
    Job(Job),
    Terminate,
}
struct Worker {
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Self {
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
    pub fn add<F>(&self, f: F)
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
