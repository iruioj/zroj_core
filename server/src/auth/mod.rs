//! Auth 模块负责用户的鉴权.
pub mod injector;

#[cfg(session_auth)]
pub mod session;

use actix_http::HttpMessage;
use actix_web::{FromRequest, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    future::{ready, Ready},
    sync::{Arc, RwLock},
};

use crate::{ClientID, UserID};

// session data for request
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AuthInfo {
    pub uid: UserID,
}

/// Extract authinfo from request-local data (cooperate with the AuthInjector middleware wrapper)
pub struct Authentication(Option<(ClientID, AuthInfo)>);

impl Authentication {
    pub fn client_id(&self) -> Option<&uuid::Uuid> {
        if let Some(c) = &self.0 {
            Some(&c.0)
        } else {
            None
        }
    }
    pub fn user_id(&self) -> Option<UserID> {
        self.0.as_ref().map(|c| c.1.uid)
    }
}

impl FromRequest for Authentication {
    type Error = actix_web::Error;
    type Future = Ready<Result<Authentication, actix_web::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_http::Payload) -> Self::Future {
        ready(Ok(
            if let Some((id, info)) = req.extensions().get::<(ClientID, AuthInfo)>() {
                Authentication(Some((id.to_owned(), info.to_owned())))
            } else {
                Authentication(None)
            },
        ))
    }
}

impl std::ops::Deref for Authentication {
    type Target = Option<(ClientID, AuthInfo)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Authentication {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct AuthStorage(pub Arc<RwLock<HashMap<ClientID, AuthInfo>>>);

impl Default for AuthStorage {
    fn default() -> Self {
        tracing::warn!("TODO: implement a LRU strategy");
        Self(Arc::new(RwLock::new(HashMap::new())))
    }
}

impl AuthStorage {
    fn get(&self, id: &ClientID) -> anyhow::Result<Option<AuthInfo>> {
        let mp = self
            .0
            .read()
            .map_err(|e| anyhow::anyhow!("query id from auth storage: {e}"))?;
        let res: Option<AuthInfo> = mp.get(id).cloned();
        Ok(res)
    }
    pub fn set(&self, id: ClientID, data: AuthInfo) -> anyhow::Result<()> {
        let mut mp = self
            .0
            .write()
            .map_err(|e| anyhow::anyhow!("modify data in auth storage: {e}"))?;
        mp.insert(id, data);
        Ok(())
    }
    pub fn remove(&self, id: &ClientID) -> anyhow::Result<()> {
        let mut mp = self
            .0
            .write()
            .map_err(|e| anyhow::anyhow!("remove data from auth storage: {e}"))?;
        mp.remove(id);
        Ok(())
    }
}

impl Clone for AuthStorage {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// name of the cookie
pub const CLIENT_ID_KEY: &str = "zroj_client_id";
