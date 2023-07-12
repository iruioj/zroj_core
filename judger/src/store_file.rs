use std::io::{self, Seek, Write, Read};

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
	/// 将文件内容读取到字符串
    pub fn read_to_string(&mut self, s: &mut String) -> Result<(), std::io::Error> {
        self.reset_cursor()?;
        self.file.read_to_string(s)?;
        Ok(())
    }
    /// 将文件内容复制到对应路径的文件
    /// 
    /// create a file if it does not exist, and will truncate it if it does.
    pub fn copy_to(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
        // dbg!(path.as_ref());
		eprintln!("test {:?}", path.as_ref());
        let mut file = std::fs::File::create(path.as_ref())?;
		eprintln!("file create");
        self.copy_all(&mut file)
    }
    /// create a temporary plain file, oftern used for testing
    pub fn create_tmp(content: impl AsRef<str>) -> Self {
        let mut file = tempfile::tempfile().expect("cannot create temporary file");
        std::io::Write::write(&mut file, content.as_ref().as_bytes())
            .expect("cannot write content to file");
        Self {
            file,
            file_type: FileType::Plain,
        }
    }
}
