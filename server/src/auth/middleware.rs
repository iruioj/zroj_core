use super::SessionManager;
use crate::auth::SessionID;
use actix_session::SessionExt;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error, web, Error, HttpMessage, Result,
};
use futures::Future;
use std::{
    future::{ready, Ready},
    pin::Pin,
};

/// SessionAuth 中间件用于从 session 中解析出 session_id
/// 并加入到 request-local data 中。
// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct SessionAuth;
// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for SessionAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SessionAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionAuthMiddleware { service }))
    }
}

pub struct SessionAuthMiddleware<S> {
    service: S,
}
impl<S> SessionAuthMiddleware<S> {
    pub fn work(&self, req: &ServiceRequest) -> Result<()> {
        let data = req
            .app_data::<web::Data<SessionManager>>()
            .ok_or(error::ErrorInternalServerError(
                "Fail to get session container",
            ))?
            .clone();
        let session = req.get_session();
        if let Some(id) = session.get::<SessionID>("session-id")? {
            eprintln!("session id = {}", id);
            if data.contains_key(id)? {
                req.extensions_mut().insert(id);
                return Ok(());
            }
        } else {
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
        println!("Hi from start. You requested: {}", req.path());
        let result = self.work(&req);
        let fut = self.service.call(req);
        Box::pin(async move {
            result?;
            fut.await
        })
    }
}
