//! file system database backend and its corresponding schemas

pub mod schema;

use anyhow::Context;
use store::{FsStore, Handle};

use super::error::DataError;

/// (Clonable) A File System database is simply a [`Handle`] of its root directory.
#[derive(Clone)]
pub struct FileSysDb(Handle);

impl FileSysDb {
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        std::fs::create_dir_all(path.as_ref())?;

        Ok(Self(Handle::new(path)))
    }
    /// remove original directory and create a new empty one
    pub fn setup_new(path: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
        if path.as_ref().exists() {
            std::fs::remove_dir_all(path.as_ref())?;
        }
        std::fs::create_dir_all(path.as_ref())?;
        Ok(())
    }

    pub fn transaction<T, F>(&self, f: F) -> Result<T, DataError>
    where
        F: FnOnce(&Handle) -> Result<T, DataError>,
    {
        f(&self.0)
    }
}

/// a string that is valid as the name of directory
///
/// ```
/// # use server::data::file_system::SanitizedString;
/// let key = SanitizedString::new("dir/hello_123").unwrap();
/// assert!(SanitizedString::new("hello 123").is_err());
/// assert!(SanitizedString::new("hello?123").is_err());
/// ```
pub struct SanitizedString(String);

#[derive(thiserror::Error, Debug)]
pub enum SanitizeError {
    #[error("invalid character {0:?} during sanitization")]
    InvalidChar(char),
    #[error("string of length {0} too long during sanitization")]
    LengthExceeded(usize),
}

impl SanitizedString {
    pub fn new(str: &str) -> Result<Self, SanitizeError> {
        let err = str
            .chars()
            .find_map(|c| {
                if c.is_ascii_control() || c.is_whitespace() || "><|:&?*".contains(c) {
                    Some(SanitizeError::InvalidChar(c))
                } else {
                    None
                }
            })
            .or(if str.len() > 128 {
                Some(SanitizeError::LengthExceeded(str.len()))
            } else {
                None
            });
        match err {
            Some(e) => Err(e),
            None => Ok(Self(str.to_string())),
        }
    }
}

impl std::fmt::Display for SanitizedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<&u32> for SanitizedString {
    type Error = SanitizeError;

    fn try_from(value: &u32) -> Result<Self, Self::Error> {
        SanitizedString::new(&value.to_string())
    }
}

impl TryFrom<&SanitizedString> for SanitizedString {
    type Error = SanitizeError;

    fn try_from(value: &SanitizedString) -> Result<Self, Self::Error> {
        Ok(Self(value.0.to_owned()))
    }
}

impl From<SanitizeError> for DataError {
    fn from(value: SanitizeError) -> Self {
        DataError::AnyError(anyhow::Error::new(value))
    }
}

/// Each table is a kv store
pub trait FileSysTable<'t>
where
    Self::Key: TryInto<SanitizedString, Error = SanitizeError>,
{
    type Key: 't;
    type Item: FsStore;

    fn ctx(&self) -> &Handle;

    fn ctx_with_key(&self, key: Self::Key) -> Result<Handle, SanitizeError> {
        let key: SanitizedString = key.try_into()?;
        Ok(self.ctx().join(key.0))
    }

    /// try to get the data
    fn query(&self, key: Self::Key) -> Result<Self::Item, DataError> {
        let ctx = self.ctx_with_key(key)?;
        Ok(Self::Item::open(&ctx)?)
    }
    /// try to get the data and its path
    fn query_with_ctx(&self, key: Self::Key) -> Result<(Self::Item, Handle), DataError> {
        let ctx = self.ctx_with_key(key)?;
        Ok((Self::Item::open(&ctx)?, ctx))
    }
    /// insert or update
    fn replace(&self, key: Self::Key, item: &'t mut Self::Item) -> Result<(), DataError> {
        let ctx = self.ctx_with_key(key)?;
        item.safe_save(&ctx)
            .with_context(|| format!("replace (upsert) ctx = {:?}", ctx.path()))?;
        Ok(())
    }
    /// update if exists
    fn update(&self, key: Self::Key, item: &'t mut Self::Item) -> Result<(), DataError> {
        let ctx = self.ctx_with_key(key)?;
        if ctx.path().exists() {
            item.safe_save(&ctx)?;
        }
        Ok(())
    }
    /// remove if exists
    fn remove(&self, key: Self::Key) -> Result<(), DataError> {
        let ctx = self.ctx_with_key(key)?;
        if ctx.path().exists() {
            ctx.remove_all()?;
        }
        Ok(())
    }
}
