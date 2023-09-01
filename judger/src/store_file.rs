use std::io::{self, Seek, Write};

use store::FsStore;

use crate::FileType;

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
