//! ZROJ 的评测模块
#![allow(dead_code)]

mod env;
mod lang;
mod one_off;
mod report;
mod store_file;
pub mod truncstr;

use std::hash::Hash;

use anyhow::Context;
// pub use cache::Cache;
pub use lang::{Compile, FileType};
pub use one_off::OneOff;
pub use report::*;
use store::FsStore;
pub use store::Handle;
pub use store_file::{SourceFile, StoreFile};
use truncstr::TruncStr;

/// Loosen the constraint of [`std::hash::Hash`],
/// for [`std::fs::File`] associated hash.
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
        Ok(Hash::hash(&self, state))
    }
}

#[derive(FsStore)]
pub struct Compilation {
    #[meta]
    pub termination: sandbox::Termination,
    #[meta]
    pub log_payload: TruncStr,
    pub execfile: std::fs::File,
}

/// Judger 是一个评测服务的上下文，可以提供评测环境的信息，访问相关缓存等等
///
/// Judger 不依赖于具体的题目类型，并且一般不会随题目评测完毕而销毁（持久化）
///
/// 写成 trait 的原因是 Judger 可以有不同的实现，例如跨平台实现、是否有缓存、是否实现了一些安全机制等等
///
/// `Judger` trait is provided for further implementation of problem task judging.
///
/// It is not recommended to create subfolder under the working directory.
pub trait Judger<
    // The type of the message (define your own message type for better performance)
    M: std::fmt::Display,
>
{
    /// 返回当前的工作目录
    fn working_dir(&self) -> store::Handle;
    /// 输出评测日志
    fn runtime_log(&mut self, msg: M);

    /// Compile `file` and generate executable at `working_dir/name`
    ///
    /// You need to ensure the source file `working_dir/name.ext` does not exist.
    fn compile(&self, file: &mut SourceFile, name: &str) -> anyhow::Result<Compilation> {
        let wd = self.working_dir();
        let src = wd.join(name).with_extension(file.file_type.ext());
        let exec = wd.join(name);
        let clog = wd.join(name).with_extension("clog");

        file.copy_all(&mut src.create_new_file()?)?;

        let term = file
            .file_type
            .compile_sandbox(&src, &exec, &clog)
            .exec_sandbox()
            .unwrap();
        Ok(Compilation {
            termination: term,
            log_payload: std::fs::read_to_string(&clog)?.into(),
            execfile: exec.open_file().context("open executable file")?,
        })
    }

    /// Copy the content of `file` to `working_dir/name`, return the handle of the destination.
    ///
    /// You need to ensure the source file `working_dir/name` does not exist.
    fn copy_store_file(
        &self,
        src: &mut StoreFile,
        name: impl AsRef<str>,
    ) -> anyhow::Result<Handle> {
        let wd = self.working_dir();
        let dest = wd.join(name.as_ref());
        src.file.safe_save(&dest)?;
        Ok(dest)
    }

    /// Copy the content of `file` to `working_dir/name`, return the handle of the destination.
    ///
    /// You need to ensure the source file `working_dir/name` does not exist.
    fn copy_file(&self, src: &mut std::fs::File, name: impl AsRef<str>) -> anyhow::Result<Handle> {
        let wd = self.working_dir();
        let dest = wd.join(name.as_ref());
        src.safe_save(&dest)?;
        Ok(dest)
    }

    /// You may reimplement this funciton to enable caching
    fn cachable_block<I: HashMut, R: FsStore>(
        &self,
        func: impl FnOnce(&Self, I) -> anyhow::Result<R>,
        inputs: I,
    ) -> anyhow::Result<R> {
        func(self, inputs)
    }
}

/// A simple judger that prints logs to `stderr`
pub struct DefaultJudger {
    wd: store::Handle,
    cached: Option<store::Handle>,
}
impl DefaultJudger {
    pub fn new(wd: store::Handle, cached: Option<store::Handle>) -> Self {
        Self { wd, cached }
    }
}
impl<M: std::fmt::Display> Judger<M> for DefaultJudger {
    fn working_dir(&self) -> store::Handle {
        self.wd.clone()
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
