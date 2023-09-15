use judger::{SourceFile, StoreFile};
use store::Handle;

use crate::RuntimeError;

/// 自动编译文件，可执行文件名为 name，编译日志为 name.c.log
pub fn compile_in_wd(
    file: &mut SourceFile,
    wd: &Handle,
    name: impl AsRef<str>,
) -> Result<judger::sandbox::Termination, RuntimeError> {
    use judger::Compile;
    let src = wd.join(String::from(name.as_ref()) + file.file_type.ext());
    let exec = wd.join(name.as_ref());
    let clog = wd.join(String::from(name.as_ref()) + ".c.log");

    file.copy_all(&mut src.create_new_file()?)?;

    let term = file
        .file_type
        .compile_sandbox(&src, &exec, &clog)
        .exec_fork()?;
    Ok(term)
}
pub fn copy_in_wd(
    file: &mut StoreFile,
    wd: &Handle,
    name: impl AsRef<str>,
) -> Result<(), RuntimeError> {
    let src = wd.join(name.as_ref());
    file.copy_all(&mut src.create_new_file()?)?;
    Ok(())
}
