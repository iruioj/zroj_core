//! file system database backend and its corresponding schemas

pub mod schema;

use store::{FsStore, Handle};

use super::error::DataError;

/// A File System database is simply a [`Handle`].
pub struct FileSysDb(Handle);

impl FileSysDb {
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        Self(Handle::new(path))
    }
    /// remove original directory and create a new one
    pub fn setup_new(path: impl AsRef<std::path::Path>) -> Result<Self, DataError> {
        if path.as_ref().exists() {
            std::fs::remove_dir_all(path.as_ref())?;
        }
        std::fs::create_dir_all(path.as_ref())?;

        Ok(Self(Handle::new(path)))
    }

    pub fn transaction<T, F>(&self, f: F) -> Result<T, DataError>
    where
        F: FnOnce(&Handle) -> Result<T, DataError>,
    {
        f(&self.0)
    }
}

/// a string that is valid as the name of directory
pub struct SanitizedString(String);

impl SanitizedString {
    pub fn new(str: &str) -> Option<Self> {
        if str
            .chars()
            .all(|c| !c.is_ascii_control() && !c.is_whitespace() && !c.is_ascii_punctuation())
        {
            Some(Self(str.to_string()))
        } else {
            None
        }
    }
}

impl From<&()> for SanitizedString {
    fn from(_: &()) -> Self {
        Self(Default::default())
    }
}

impl From<&u32> for SanitizedString {
    fn from(value: &u32) -> Self {
        Self(value.to_string())
    }
}

/// Each table is a kv store
pub trait FileSysTable
where
    for<'a> &'a Self::Key: Into<SanitizedString>,
{
    type Key;
    type Item: FsStore;

    fn ctx(&self) -> &Handle;

    /// try to get the data
    fn query(&self, key: &Self::Key) -> Result<Self::Item, DataError> {
        let ctx = self.ctx().join(Into::<SanitizedString>::into(key).0);
        Ok(Self::Item::open(&ctx)?)
    }
    /// insert or update
    fn replace(&self, key: &Self::Key, item: &mut Self::Item) -> Result<(), DataError> {
        let ctx = self.ctx().join(Into::<SanitizedString>::into(key).0);
        item.safe_save(&ctx)?;
        Ok(())
    }
    /// update if exists
    fn update(&self, key: &Self::Key, item: &mut Self::Item) -> Result<(), DataError> {
        let ctx = self.ctx().join(Into::<SanitizedString>::into(key).0);
        if ctx.path().exists() {
            item.safe_save(&ctx)?;
        }
        Ok(())
    }
    /// remove if exists
    fn remove(&self, key: &Self::Key) -> Result<(), DataError> {
        let ctx = self.ctx().join(Into::<SanitizedString>::into(key).0);
        if ctx.path().exists() {
            ctx.remove_all()?;
        }
        Ok(())
    }
}
