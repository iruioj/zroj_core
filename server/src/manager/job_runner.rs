pub type Job = Box<dyn FnOnce() + Send + Sync + 'static>;

/// A general job runner, which spawns a new thread for job running
pub struct JobRunner {
    sender: crossbeam_channel::Sender<Job>,
    handle: std::thread::JoinHandle<()>,
}
impl JobRunner {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<Job>();

        let handle = std::thread::spawn(move || {
            eprintln!("[job] thread start");
            loop {
                match receiver.recv() {
                    Ok(job) => {
                        eprintln!("[job] receive new job");
                        job();
                    }
                    Err(_) => {
                        // closed
                        eprintln!("[job] close job thread");
                        return;
                    }
                }
            }
        });
        Self { sender, handle }
    }
    /// on error: message could not be sent because the channel is disconnected.
    pub fn add_job<F>(&self, f: F) -> Result<(), crossbeam_channel::SendError<Job>>
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        let job = Box::new(f);
        self.sender
            .send(job)
    }
    /// wait for job thread to finish and then close it
    pub fn terminate(self) {
        drop(self.sender); // explicitly drop sender to close job thread
        self.handle.join().unwrap();
    }
}