use super::SessionManager;
use crate::auth::{SessionData, SessionID};
use actix_session::SessionExt;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error, web, Error, HttpMessage, Result,
};
use futures::Future;
use std::{
    future::{ready, Ready},
    pin::Pin,
    sync::Arc,
};

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct RequireSession;
// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for RequireSession
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireSessionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireSessionMiddleware { service }))
    }
}

pub struct RequireSessionMiddleware<S> {
    service: S,
}
impl<S> RequireSessionMiddleware<S> {
    pub fn work(&self, req: &ServiceRequest) -> Result<()> {
        let data = req
            .app_data::<web::Data<Arc<SessionManager>>>()
            .ok_or(error::ErrorInternalServerError(
                "Fail to get session container",
            ))?
            .clone();
        let session = req.get_session();
        if let Some(id) = session.get::<SessionID>("session-id")? {
            if data.contains_key(id)? {
                req.extensions_mut().insert(id);
                return Ok(());
            }
        }
        let id = SessionID::new_v4(); // generate a random session-id
        data.set(id, SessionData { login_state: None })?;
        req.extensions_mut().insert(id);
        Ok(())
    }
}
impl<S, B> Service<ServiceRequest> for RequireSessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    // type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        println!("Hi from start. You requested: {}", req.path());
        let result = self.work(&req);
        let fut = self.service.call(req);
        Box::pin(async move {
            result?;
            fut.await
        })
    }
}

pub struct RequireLogin;
impl<S, B> Transform<S, ServiceRequest> for RequireLogin
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireLoginMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireLoginMiddleware { service }))
    }
}

pub struct RequireLoginMiddleware<S> {
    service: S,
}
impl<S> RequireLoginMiddleware<S> {
    pub fn work(&self, req: &ServiceRequest) -> Result<()> {
        let data = req
            .app_data::<SessionManager>()
            .ok_or(error::ErrorInternalServerError(
                "Fail to get session container",
            ))?;
        let id = req
            .extensions()
            .get::<SessionID>()
            .ok_or(error::ErrorInternalServerError(
                "Session id is required for login middleware",
            ))?
            .clone();
        let state = data
            .get(id.clone())?
            .ok_or(error::ErrorInternalServerError(
                "Login middleware: No session data for this session id",
            ))?;
        match state.login_state {
            None => Err(error::ErrorUnauthorized("You must login first")),
            Some(uid) => {
                req.extensions_mut().insert(uid);
                Ok(())
            }
        }
    }
}
impl<S, B> Service<ServiceRequest> for RequireLoginMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    // type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

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
