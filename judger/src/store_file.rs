use crate::FileType;
use std::{
    io::{self, BufReader, Read, Seek, Write},
    string::FromUtf8Error,
};
use store::FsStore;

/// 一个带类型的 buffer
#[derive(Debug)]
pub struct StoreBytes {
    pub buf: Vec<u8>,
    pub file_type: FileType,
}

impl StoreBytes {
    /// get utf-8 text content
    pub fn utf8(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.buf.clone())
    }
    pub fn from_str(content: impl AsRef<str>, file_type: FileType) -> Self {
        let buf: Vec<u8> = content.as_ref().as_bytes().to_vec();
        Self { buf, file_type }
    }
    pub fn copy_all(&mut self, dest: &mut impl Write) -> Result<(), std::io::Error> {
        dest.write(&self.buf)?;
        Ok(())
    }
    /// 将内容复制到对应路径的文件
    pub fn copy_to(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
        let mut file = std::fs::File::create(path.as_ref())?;
        self.copy_all(&mut file)
    }
}

impl FsStore for StoreBytes {
    fn open(ctx: &store::Handle) -> Result<Self, store::Error> {
        let mut reader = BufReader::new(ctx.join("buf").open_file()?);
        let mut buf = Vec::new();
        reader
            .read_to_end(&mut buf)
            .map_err(store::Error::OpenFile)?;
        let file_type = ctx.join("file_type").deserialize()?;
        Ok(Self { buf, file_type })
    }

    fn save(&mut self, ctx: &store::Handle) -> Result<(), store::Error> {
        ctx.join("file_type").serialize_new_file(&self.file_type)?;
        let mut dest = ctx.join("buf").create_new_file()?;
        dest.write(&self.buf).expect("writing buf to file");
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
    pub fn reset_cursor(&mut self) -> Result<(), std::io::Error> {
        self.file.seek(io::SeekFrom::Start(0))?;
        Ok(())
    }
    pub fn copy_all(&mut self, dest: &mut impl Write) -> Result<(), std::io::Error> {
        self.reset_cursor()?;
        std::io::copy(&mut self.file, dest)?;
        Ok(())
    }
    /// 将文件内容复制到对应路径的文件
    ///
    /// create a file if it does not exist, and will truncate it if it does.
    pub fn copy_to(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
        // dbg!(path.as_ref());
        let mut file = std::fs::File::create(path.as_ref())?;
        self.copy_all(&mut file)
    }
    /// create a temporary file with corresponding file_type
    pub fn from_str(content: impl AsRef<str>, file_type: FileType) -> Self {
        let mut file = tempfile::tempfile().expect("create tmp file");
        std::io::Write::write(&mut file, content.as_ref().as_bytes())
            .expect("cannot write content to file");
        Self { file, file_type }
    }
    pub fn read_to_string(&mut self) -> Result<String, std::io::Error> {
        let mut buf = io::BufWriter::new(Vec::new());
        self.copy_all(&mut buf)?;
        Ok(String::from_utf8(buf.into_inner()?).expect("text encoding should be utf8"))
    }
}
