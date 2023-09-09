use serde_ts_typing::TypeId;
use std::{
    any::Any,
    collections::BTreeMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use store::{FsStore, Handle};

use super::error::DataError;

pub struct FsStoreLock<V>
where
    V: FsStore + Default,
{
    data: RwLock<V>,
    pub ctx: Handle,
}

// make sure no FsStoreWriteGuard write at the same time
lazy_static::lazy_static! {
    static ref FS_STORE_CACHE: Mutex<BTreeMap<TypeId, Arc<dyn Any + Send + Sync>>> = Default::default();
}

// This implementation is not elegant. Just for convenience.
pub struct FsStoreWriteGuard<'a, V>(RwLockWriteGuard<'a, V>, &'a Handle)
where
    V: FsStore;

impl<'a, V> Drop for FsStoreWriteGuard<'a, V>
where
    V: FsStore,
{
    fn drop(&mut self) {
        let v = self.0.deref_mut();
        v.save(self.1).expect("value should be saved");
    }
}

impl<'a, V> Deref for FsStoreWriteGuard<'a, V>
where
    V: FsStore,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<'a, V> DerefMut for FsStoreWriteGuard<'a, V>
where
    V: FsStore,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl<V> FsStoreLock<V>
where
    V: FsStore + Default + 'static,
{
    /// load `path/{}`
    fn try_load(ctx: Handle) -> Result<Self, store::Error> {
        let data = if let Ok(r) = V::open(&ctx) {
            r
        } else {
            ctx.remove_all()?;
            V::default()
        };

        Ok(Self {
            data: data.into(),
            ctx,
        })
    }
    pub fn read(&self) -> Result<RwLockReadGuard<'_, V>, PoisonError<RwLockReadGuard<'_, V>>> {
        self.data.read()
    }
    pub fn write(&self) -> Result<FsStoreWriteGuard<'_, V>, PoisonError<RwLockWriteGuard<'_, V>>> {
        Ok(FsStoreWriteGuard(self.data.write()?, &self.ctx))
    }
}

pub struct FsStoreDb(Handle);

pub struct FsStoreTable<V>(Handle, PhantomData<V>);

impl<V> FsStoreTable<V>
where
    V: FsStore + Default + 'static + Send + Sync,
{
    fn init_table(&self) -> Result<FsStoreLock<V>, DataError> {
        let tid = TypeId::of::<V>();
        let hash = judger::seq_hash!(format!("{:?}", tid));
        Ok(FsStoreLock::try_load(self.0.join(hash))?)
    }
    pub fn get_table(&self) -> Result<Arc<FsStoreLock<V>>, DataError> {
        let tid = TypeId::of::<V>();
        let mut cache = FS_STORE_CACHE.lock()?;
        let r = cache
            .entry(tid)
            .or_insert(Arc::new(self.init_table()?))
            .clone();
        let val = r
            .downcast::<FsStoreLock<V>>()
            .expect("cache value of key type V should be of type Arc<FsStoreLock<V>>");
        Ok(val)
    }
    /// imitate [`diesel::connection::Connection::transaction`]
    pub fn read_transaction<R, F>(&self, f: F) -> Result<R, DataError>
    where
        F: FnOnce(&V) -> Result<R, DataError>,
    {
        let guard = self.get_table()?;
        let read = guard.read()?;
        f(read.deref())
    }
    /// imitate [`diesel::connection::Connection::transaction`]
    pub fn write_transaction<R, F>(&self, f: F) -> Result<R, DataError>
    where
        F: FnOnce(&mut V) -> Result<R, DataError>,
    {
        let guard = self.get_table()?;
        let mut read = guard.write()?;
        f(read.deref_mut())
    }
}

impl FsStoreDb {
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        let ctx = Handle::new(path.as_ref());
        Self(ctx)
    }
    pub fn table<V>(&self) -> FsStoreTable<V>
    where
        V: FsStore + Default + 'static + Send + Sync,
    {
        FsStoreTable(self.0.clone(), PhantomData)
    }
}
