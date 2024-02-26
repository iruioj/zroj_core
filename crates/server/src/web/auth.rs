/*!
Provide user authentication utilities.

A user accesses the resources on the server from whatever devices must send requests
containing certain credential (e.g. a UUID), which is used by server to obtain its
identity information. This identity will be further used to check his/her permission
to get the corresponding resources.
*/

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error, Error, FromRequest, HttpMessage,
};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    future::{ready, Ready},
    pin::Pin,
    rc::Rc,
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
    pub fn user_id_or_unauthorized(&self) -> Result<UserID, actix_web::Error> {
        self.user_id()
            .ok_or(actix_web::error::ErrorUnauthorized("user_id not found"))
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

/// Stores the map from a client's credential (which is currently a UUID) to his/her
/// identity information.
pub struct AuthStorage(Arc<RwLock<HashMap<ClientID, AuthInfo>>>);

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
    fn set(&self, id: ClientID, data: AuthInfo) -> anyhow::Result<()> {
        let mut mp = self
            .0
            .write()
            .map_err(|e| anyhow::anyhow!("modify data in auth storage: {e}"))?;
        mp.insert(id, data);
        Ok(())
    }
    fn remove(&self, id: &ClientID) -> anyhow::Result<()> {
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
const CLIENT_ID_KEY: &str = "zroj_client_id";

/// Add manipulation to response-local data to update [`AuthStorage`].
pub enum Manip {
    Insert(AuthInfo),
    Delete(ClientID),
}

struct Inner {
    // 是否要求有鉴权信息。如果没有，返回 401 Unauthorized
    require: bool,
    store: AuthStorage,
}

/// AuthInjector is a middleware that tries to extracts authentication data
/// and register them into request-local data for future use.
///
/// See [`super::Authentication`] for extractor usage.
///
/// # Example
///
/// ```rust
/// # use server::auth::injector::AuthInjector;
/// # use server::auth::AuthStorage;
/// let auth_storage = AuthStorage::default();
///
/// actix_web::App::new()
///     .wrap(AuthInjector::require_auth(auth_storage.clone()));
/// ```
pub struct AuthInjector(Rc<Inner>);

impl AuthInjector {
    /// Bypass all reqests, trying to extract identity to request-local data.
    pub fn bypass(store: AuthStorage) -> Self {
        Self(Rc::new(Inner {
            require: false,
            store,
        }))
    }
    /// If identity not found, 401 unauthorized error is returned
    pub fn require_auth(store: AuthStorage) -> Self {
        Self(Rc::new(Inner {
            require: true,
            store,
        }))
    }
}

impl Clone for AuthInjector {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
impl<S, B> Transform<S, ServiceRequest> for AuthInjector
where
    // `S` - type of the next service
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    // `B` - type of response's body
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = Middleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(Middleware {
            service,
            inner: self.0.clone(),
        }))
    }
}

#[doc(hidden)]
pub struct Middleware<S> {
    service: S,
    inner: Rc<Inner>,
}
impl<S> Middleware<S> {
    fn extract_info(&self, req: &ServiceRequest) -> anyhow::Result<(ClientID, AuthInfo)> {
        let c = req
            .cookie(CLIENT_ID_KEY)
            .ok_or(anyhow::anyhow!("cookie name `{CLIENT_ID_KEY}` not found"))?;
        let id = uuid::Uuid::parse_str(c.value()).context("parse uuid")?;
        let info = self
            .inner
            .store
            .get(&id)?
            .ok_or(anyhow::anyhow!("authinfo not found"))?;
        Ok((id, info))
    }
    pub fn work(&self, req: &ServiceRequest) -> actix_web::Result<()> {
        match self.extract_info(req) {
            Ok((id, info)) => {
                tracing::debug!("client id = {id}");
                req.extensions_mut().insert((id, info));
            }
            Err(e) => {
                if self.inner.require {
                    return Err(error::ErrorUnauthorized(e));
                }
            }
        }
        Ok(())
    }
}
impl<S, B> Service<ServiceRequest> for Middleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>>>>;
    forward_ready!(service);
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let result = self.work(&req);
        let fut = self.service.call(req);
        let inner = self.inner.clone();

        Box::pin(async move {
            result?;
            let mut r = fut.await?;

            let op = r.response_mut().extensions_mut().remove::<Manip>();
            if let Some(op) = op {
                match op {
                    Manip::Insert(info) => {
                        let id = if cfg!(feature = "uid_as_cid") {
                            // used for request recording
                            tracing::info!("generate client id from uid");
                            ClientID::from_u128(info.uid as u128)
                        } else {
                            ClientID::new_v4() // generate a random session id
                        };

                        tracing::info!("generate new client id {id}");

                        r.response_mut().add_cookie(
                            &actix_web::cookie::Cookie::build(CLIENT_ID_KEY, id.to_string())
                                .path("/")
                                .finish(),
                        )?;
                        inner.store.set(id, info)
                    }
                    Manip::Delete(client_id) => inner.store.remove(&client_id),
                }
                .map_err(error::ErrorInternalServerError)?;
            }
            Ok(r)
        })
    }
}
