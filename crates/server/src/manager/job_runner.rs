pub type Job = Box<dyn FnOnce() + Send + Sync + 'static>;

/// A general job runner, which spawns a new thread for job running.
///
/// To close the channel and detach the job thread, simply drop it.
///
/// ```rust
#[doc = include_str!("../../examples/job_runner.rs")]
/// ```
pub struct JobRunner {
    // channel is closed after dropping the sender
    sender: crossbeam_channel::Sender<Job>,
    // thread is detached after dropping the handle
    handle: std::thread::JoinHandle<()>,
}
impl Default for JobRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl JobRunner {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<Job>();

        let handle = std::thread::spawn(move || {
            tracing::info!("job thread start");
            loop {
                match receiver.recv() {
                    Ok(job) => {
                        tracing::debug!("receive new job");
                        job();
                    }
                    Err(_) => {
                        // closed
                        tracing::info!("close job thread");
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
        self.sender.send(job)
    }
    /// drop the sender to close the channel, and join the job thread
    pub fn terminate_join(self) -> Result<(), Box<dyn std::any::Any + Send>> {
        drop(self.sender);
        self.handle.join()
    }
}
