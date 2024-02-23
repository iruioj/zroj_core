//! ZROJ 的评测模块
#![allow(dead_code)]

mod env;
mod lang;
mod one_off;
mod report;
mod store_file;
pub mod truncstr;

use std::{hash::Hash, marker::PhantomData, process::Stdio};

use anyhow::Context;
// pub use cache::Cache;
use ::sandbox::{unix::SingletonConfig, Termination};
pub use env::which;
pub use lang::{FileType, COMPILE_LIM};
pub use one_off::OneOff;
pub use report::*;
use store::FsStore;
pub use store::Handle;
pub use store_file::{SourceFile, StoreFile};
use truncstr::TruncStr;

/// Loosen the constraint of [`std::hash::Hash`],
/// for [`std::fs::File`] associated hash.
///
/// See [`Judger::cachable_block`] for usage.
pub trait HashMut {
    fn hash_mut<H: std::hash::Hasher>(&mut self, state: &mut H) -> anyhow::Result<()>;
}

impl HashMut for StoreFile {
    fn hash_mut<H: std::hash::Hasher>(&mut self, state: &mut H) -> anyhow::Result<()> {
        state.write(&self.read_to_bytes()?);
        Ok(())
    }
}

impl<T: Hash> HashMut for T {
    fn hash_mut<H: std::hash::Hasher>(&mut self, state: &mut H) -> anyhow::Result<()> {
        Hash::hash(&self, state);
        Ok(())
    }
}

#[derive(FsStore)]
pub struct Compilation {
    #[meta]
    pub termination: sandbox::Termination,
    #[meta]
    pub log_payload: TruncStr,
    pub execfile: Option<std::fs::File>,
}

/// `Judger` handles the execution of task (testcase) judgement. It provides working directory information,
/// FS manipulation utilities and an optional cache backend.
///
/// `Judger` does not depend on specific problem type, and is often not dropped after each testcase done.
///
/// It is not recommended to create subfolder under the working directory.
pub trait Judger<M: std::fmt::Display> {
    /// return the current working directory.
    fn working_dir(&self) -> &store::Handle;
    /// write judger log, where `M` is the type of message,
    /// which implements [`std::fmt::Display`].
    /// Define your own message type for better performance.
    fn runtime_log(&mut self, msg: M);

    /// This method will do the following things:
    ///
    /// 1. create a source file at `working_dir/name.{ext}`;
    /// 2. compile it and generate executable at `working_dir/name`.
    ///
    /// You need to ensure the source file `working_dir/name.ext` does not exist.
    /// If you're applying [`Judger::cachable_block`], you should use [`Compilation::execfile`] for
    /// further processing.
    fn compile(&self, file: &mut SourceFile, name: &str) -> anyhow::Result<Compilation> {
        let wd = self.working_dir();
        let src = wd.join(name).with_extension(file.file_type.ext());
        let exec = wd.join(name);
        let clog = wd.join(name).with_extension("clog");

        file.copy_all(&mut src.create_new_file()?)?;

        let term = self
            .exec_sandbox(file.file_type.compile_sandbox(&src, &exec, &clog))
            .context("compile file")?;
        Ok(Compilation {
            termination: term,
            log_payload: std::fs::read_to_string(&clog)?.into(),
            execfile: exec.open_file().ok(),
        })
    }

    /// Copy the content of `file` to `working_dir/name`, return the handle of the destination.
    ///
    /// You need to ensure the source file `working_dir/name` does not exist.
    fn copy_store_file(&self, src: &mut StoreFile, name: &str) -> anyhow::Result<Handle> {
        let wd = self.working_dir();
        let dest = wd.join(name);
        src.file.safe_save(&dest)?;
        Ok(dest)
    }

    fn create_source_file(&self, content: &str, name: &str) -> anyhow::Result<Handle> {
        let wd = self.working_dir();
        let dest = wd.join(name);
        let mut f = dest.create_new_file()?;
        use std::io::Write;
        f.write_all(content.as_bytes())?;
        Ok(dest)
    }

    /// Copy the content of `file` to `working_dir/name`, return the handle of the destination.
    ///
    /// You need to ensure the source file `working_dir/name` does not exist.
    fn copy_file(&self, src: &mut std::fs::File, name: &str) -> anyhow::Result<Handle> {
        let wd = self.working_dir();
        let dest = wd.join(name);
        src.safe_save(&dest)?;
        Ok(dest)
    }

    /// remove `working_dir/name` and return its path
    fn clear_dest(&self, name: &str) -> anyhow::Result<Handle> {
        let wd = self.working_dir();
        let path = wd.join(name);
        path.remove_all()?;
        Ok(path)
    }

    /// You may reimplement this funciton to enable caching
    fn cachable_block<I: HashMut, R: FsStore>(
        &self,
        func: impl FnOnce(&Self, I) -> anyhow::Result<R>,
        inputs: I,
    ) -> anyhow::Result<R> {
        func(self, inputs)
    }

    /// Use the `zroj-sandbox` to execute
    fn exec_sandbox(&self, cfg: SingletonConfig) -> anyhow::Result<Termination> {
        let mut child = std::process::Command::new("zroj-sandbox")
            .arg("run")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("failed to spawn child process")?;

        let stdin = child.stdin.take().context("failed to open stdin")?;
        std::thread::spawn(move || {
            serde_json::to_writer(stdin, &cfg).expect("failed to write to stdin");
        });
        let output = child.wait_with_output().context("failed to read stdout")?;
        let term: Result<Termination, Vec<String>> =
            serde_json::from_slice(&output.stdout).context("deserialize sandbox output")?;

        term.map_err(|e| anyhow::anyhow!("sandbox error: {e:?}"))
    }
}

/// A simple judger that prints logs to `stderr`.
pub struct DefaultJudger<M> {
    wd: store::Handle,
    cached: Option<store::Handle>,
    _mark: PhantomData<M>,
}
impl<M> DefaultJudger<M> {
    pub fn new(wd: store::Handle, cached: Option<store::Handle>) -> Self {
        Self {
            wd,
            cached,
            _mark: PhantomData,
        }
    }
}
impl<M: std::fmt::Display> Judger<M> for DefaultJudger<M> {
    fn working_dir(&self) -> &store::Handle {
        &self.wd
    }
    fn runtime_log(&mut self, msg: M) {
        eprintln!("[judger] {}", msg)
    }
    /// implement a simple fs cache
    fn cachable_block<I: HashMut, R: FsStore>(
        &self,
        func: impl FnOnce(&Self, I) -> anyhow::Result<R>,
        mut inputs: I,
    ) -> anyhow::Result<R> {
        if let Some(cache_root) = &self.cached {
            let h = {
                let mut s = std::hash::DefaultHasher::new();
                inputs.hash_mut(&mut s).context("calcuate input hash")?;
                std::hash::Hasher::finish(&s)
            };
            let path = cache_root.join(h.to_string());
            if path.path().exists() {
                if let Ok(r) = R::open(&path) {
                    eprintln!("[judger] find cache {:?}", path.path());
                    return Ok(r);
                }
                eprintln!(
                    "[judger] find cache but fail to deserialize {:?}",
                    path.path()
                );
            } else {
                eprintln!("[judger] not find cache {:?}", path.path());
            }
            let mut r = func(self, inputs)?;
            r.save(&path)?;
            assert!(path.path().exists());
            eprintln!("[judger] add cache {:?}", path.path());
            Ok(r)
        } else {
            func(self, inputs)
        }
    }
}

// re-export
pub mod sandbox {
    pub use sandbox::*;
}
