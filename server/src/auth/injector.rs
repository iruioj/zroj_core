use super::{AuthInfo, AuthStorage, CLIENT_ID_KEY};
use crate::ClientID;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error, Error, HttpMessage, Result,
};
use anyhow::Context;
use futures::Future;
use std::{
    future::{ready, Ready},
    pin::Pin,
    rc::Rc,
};

// AuthInjector 的内部数据
struct Inner {
    // 是否要求有鉴权信息。如果没有，返回 401 Unauthorized
    require: bool,
    store: AuthStorage,
}

/// AuthInjector is a middleware that tries to extracts authentication data
/// and register them into request-local data for future use.
///
/// This middleware relies on the [`actix_session`] crate since it extracts auth. info.
/// from session.
///
/// Usage:
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
    /// 放过所有请求，只尝试提取鉴权信息
    pub fn bypass(store: AuthStorage) -> Self {
        Self(Rc::new(Inner {
            require: false,
            store,
        }))
    }
    /// 要求必须具有鉴权信息，否则返回 401 Unauthorized
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
    type Transform = AuthInjectorMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthInjectorMiddleware {
            service,
            inner: self.0.clone(),
        }))
    }
}

#[doc(hidden)]
pub struct AuthInjectorMiddleware<S> {
    service: S,
    inner: Rc<Inner>,
}
impl<S> AuthInjectorMiddleware<S> {
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
    pub fn work(&self, req: &ServiceRequest) -> Result<()> {
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
impl<S, B> Service<ServiceRequest> for AuthInjectorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    forward_ready!(service);
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let result = self.work(&req);
        let fut = self.service.call(req);
        Box::pin(async move {
            result?;
            fut.await
        })
    }
}
