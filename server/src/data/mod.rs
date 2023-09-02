pub mod error;
#[cfg(feature = "mysql")]
mod mysql;
pub mod types;

// database
pub mod gravatar;
pub mod problem_ojdata;
pub mod problem_statement;
pub mod submission;
pub mod user;

// pub mod group;
// pub mod problem_config;
// pub mod schema;

/// 定义一个类型为 web::Data<ty> 的值
#[macro_export]
macro_rules! mkdata {
    ($t:ty, $e:expr) => {
        actix_web::web::Data::from(std::sync::Arc::new($e) as std::sync::Arc<$t>)
    };
}
use std::{
    ops::{Deref, DerefMut},
    sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub use mkdata;
use store::{FsStore, Handle};

// fn notfound_as_none<T>(r: Result<T, error::DataError>) -> Result<Option<T>, error::DataError> {
//     match r {
//         Ok(t) => Ok(Some(t)),
//         Err(e) => match e {
//             error::DataError::NotFound => Ok(None),
//             #[cfg(feature = "mysql")]
//             error::DataError::Diesel(diesel::result::Error::NotFound) => Ok(None),
//             e => Err(e),
//         },
//     }
// }

pub struct FsStoreLock<V>
where
    V: FsStore + Default,
{
    data: RwLock<V>,
    ctx: Handle,
}

// This implementation is not elegant. Just for convenience.
pub struct FsStoreLockWriteGuard<'a, V>(RwLockWriteGuard<'a, V>, &'a Handle)
where
    V: FsStore;

impl<'a, V> Drop for FsStoreLockWriteGuard<'a, V>
where
    V: FsStore,
{
    fn drop(&mut self) {
        let v = self.0.deref_mut();
        v.save(self.1).expect("value should be saved")
    }
}

impl<'a, V> Deref for FsStoreLockWriteGuard<'a, V>
where
    V: FsStore,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<'a, V> DerefMut for FsStoreLockWriteGuard<'a, V>
where
    V: FsStore,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl<V> FsStoreLock<V>
where
    V: FsStore + Default,
{
    pub fn try_load(path: impl AsRef<std::path::Path>) -> Result<Self, store::Error> {
        let ctx = Handle::new(path.as_ref());
        let data = if let Ok(r) = V::open(&ctx) {
            r
        } else {
            ctx.remove_all()?;
            V::default()
        };

        Ok(Self { data: data.into(), ctx })
    }
    pub fn read(&self) -> Result<RwLockReadGuard<'_, V>, PoisonError<RwLockReadGuard<'_, V>>> {
        self.data.read()
    }
    pub fn write(
        &self,
    ) -> Result<FsStoreLockWriteGuard<'_, V>, PoisonError<RwLockWriteGuard<'_, V>>> {
        Ok(FsStoreLockWriteGuard(self.data.write()?, &self.ctx))
    }
}
