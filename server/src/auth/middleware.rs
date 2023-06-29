use super::SessionManager;
use crate::auth::SessionID;
use actix_session::SessionExt;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error,
    Error, HttpMessage, Result,
};
use futures::Future;
use std::{
    future::{ready, Ready},
    pin::Pin,
    rc::Rc,
};

// SessionAuth 的内部数据
struct Inner {
    // 是否要求有鉴权信息。如果没有，返回 401 Unauthorized
    require: bool,
    store: SessionManager,
}

/// SessionAuth 中间件用于从 session 中（尽可能）解析出 SessionID
/// 和 UserId 并加入到 request-local data 中。
pub struct SessionAuth(Rc<Inner>);

impl SessionAuth {
    /// 放过所有请求，只尝试提取鉴权信息
    pub fn bypass(store: SessionManager) -> Self {
        Self(Rc::new(Inner {
            require: false,
            store,
        }))
    }
    /// 要求必须具有鉴权信息，否则返回 401 Unauthorized
    pub fn require_auth(store: SessionManager) -> Self {
        Self(Rc::new(Inner {
            require: true,
            store,
        }))
    }
}

impl Clone for SessionAuth {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
impl<S, B> Transform<S, ServiceRequest> for SessionAuth
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
    type Transform = SessionAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionAuthMiddleware {
            service,
            inner: self.0.clone(),
        }))
    }
}

#[doc(hidden)]
pub struct SessionAuthMiddleware<S> {
    service: S,
    inner: Rc<Inner>,
}
impl<S> SessionAuthMiddleware<S> {
    pub fn work(&self, req: &ServiceRequest) -> Result<()> {
        let session = req.get_session();
        if let Some(id) = session.get::<SessionID>(super::SESSION_ID_KEY)? {
            eprintln!("[auth middleware] session id = {}", id);
            if let Some(info) = self.inner.store.get(id)? {
                req.extensions_mut().insert(id);
                req.extensions_mut().insert(info.uid);
                return Ok(());
            } else if self.inner.require {
                // has session id but info not found
                session.remove(super::SESSION_ID_KEY);
                return Err(error::ErrorUnauthorized("invalid session id"));
            }
        } else {
            // no session id
            if self.inner.require {
                return Err(error::ErrorUnauthorized("not login"));
            }
            eprintln!("no session id found");
            dbg!(req.cookies().unwrap());
        }
        Ok(())
    }
}
impl<S, B> Service<ServiceRequest> for SessionAuthMiddleware<S>
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
