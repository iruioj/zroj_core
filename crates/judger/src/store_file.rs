use crate::FileType;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;
use std::io::{self, Seek, Write};
use store::FsStore;

/// 一个带类型的 buffer
#[derive(Debug, Serialize, Deserialize, TsType, Clone, Hash)]
pub struct SourceFile {
    pub source: String,
    pub file_type: FileType,
}

impl SourceFile {
    /// get utf-8 text content
    pub fn utf8(&self) -> String {
        self.source.clone()
    }
    pub fn from_str(content: impl AsRef<str>, file_type: FileType) -> Self {
        let source = content.as_ref().to_string();
        Self { source, file_type }
    }
    pub fn copy_all(&mut self, dest: &mut impl Write) -> Result<(), io::Error> {
        dest.write_all(self.source.as_bytes())?;
        Ok(())
    }
    /// 将内容复制到对应路径的文件
    pub fn copy_to(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), io::Error> {
        let mut file = std::fs::File::create(path.as_ref())?;
        self.copy_all(&mut file)
    }
}

impl FsStore for SourceFile {
    fn open(ctx: &store::Handle) -> Result<Self, store::Error> {
        let source =
            std::io::read_to_string(ctx.join("buf").open_file()?).context("open source file")?;
        let file_type = ctx.join("file_type").deserialize()?;
        Ok(Self { source, file_type })
    }

    fn save(&mut self, ctx: &store::Handle) -> Result<(), store::Error> {
        ctx.join("file_type").serialize_new_file(&self.file_type)?;
        let mut dest = ctx.join("buf").create_new_file()?;
        dest.write_all(self.source.as_bytes())
            .expect("writing buf to file");
        Ok(())
    }
}

/// 一个带类型的文件
#[derive(FsStore, Debug)]
pub struct StoreFile {
    pub file: std::fs::File,
    #[meta]
    pub file_type: FileType,
}

impl StoreFile {
    pub fn reset_cursor(&mut self) -> Result<(), io::Error> {
        self.file.seek(io::SeekFrom::Start(0))?;
        Ok(())
    }
    pub fn copy_all(&mut self, dest: &mut impl Write) -> Result<(), io::Error> {
        self.reset_cursor()?;
        std::io::copy(&mut self.file, dest)?;
        Ok(())
    }
    /// 将文件内容复制到对应路径的文件
    ///
    /// create a file if it does not exist, and will truncate it if it does.
    pub fn copy_to(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), io::Error> {
        // dbg!(path.as_ref());
        let mut file = std::fs::File::create(path.as_ref())?;
        self.copy_all(&mut file)
    }
    /// create a temporary file with corresponding file_type and set file position
    /// to the starting point
    pub fn from_str(content: impl AsRef<str>, file_type: FileType) -> Self {
        let mut file = tempfile::tempfile().expect("create tmp file");
        std::io::Write::write(&mut file, content.as_ref().as_bytes())
            .expect("cannot write content to file");
        file.seek(io::SeekFrom::Start(0)).expect("move to the start of file");
        Self { file, file_type }
    }
    /// read (from start) the whole content to byte array
    pub fn read_to_bytes(&mut self) -> Result<Vec<u8>, io::Error> {
        let mut buf = io::BufWriter::new(Vec::new());
        self.copy_all(&mut buf)?;
        Ok(buf.into_inner()?)
    }
    /// read (from start) the whole content to string
    pub fn read_to_string(&mut self) -> anyhow::Result<String> {
        Ok(String::from_utf8(self.read_to_bytes()?).context("text encoding should be utf8")?)
    }
}
