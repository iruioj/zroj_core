//! Auth 模块负责用户的鉴权.
pub mod middleware;
use actix_web::{
    error::{self},
    Result,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{SessionID, UserID};

// session data for request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthInfo {
    pub uid: UserID,
}

/// session data container
pub struct SessionManager(pub Arc<RwLock<HashMap<SessionID, AuthInfo>>>);
impl SessionManager {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::<SessionID, AuthInfo>::new())))
    }
    pub fn get(&self, id: SessionID) -> Result<Option<AuthInfo>> {
        let mp = self
            .0
            .read()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        let res: Option<AuthInfo> = mp.get(&id).cloned();
        Ok(res)
    }
    pub fn set(&self, id: SessionID, data: AuthInfo) -> Result<()> {
        let mut mp = self
            .0
            .write()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        mp.insert(id, data);
        Ok(())
    }
    pub fn remove(&self, id: SessionID) -> Result<()> {
        let mut mp = self
            .0
            .write()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        mp.remove(&id);
        Ok(())
    }
    pub fn contains_key(&self, id: SessionID) -> Result<bool> {
        let mp = self
            .0
            .read()
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
        Ok(mp.contains_key(&id))
    }
}

impl From<Arc<RwLock<HashMap<SessionID, AuthInfo>>>> for SessionManager {
    fn from(value: Arc<RwLock<HashMap<SessionID, AuthInfo>>>) -> Self {
        Self(value)
    }
}
impl Clone for SessionManager {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub const SESSION_ID_KEY: &'static str = "session-id";