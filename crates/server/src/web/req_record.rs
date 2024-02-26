/*!
This middleware records the HTTP requests for testing
*/

use actix_http::{BoxedPayloadStream, Payload};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error, Error, HttpMessage,
};
use anyhow::Context;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    future::{ready, Ready},
    ops::Sub,
    pin::Pin,
    rc::Rc,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

pub struct ReqRecord(Arc<Inner>);

impl Clone for ReqRecord {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl ReqRecord {
    pub fn new(dir: &store::Handle, start_time: &DateTime<Utc>) -> Self {
        Self(Arc::new(Inner {
            dir: dir.clone(),
            start_time: *start_time,
            count: AtomicU32::new(0),
        }))
    }
}

struct Inner {
    count: AtomicU32,
    start_time: DateTime<Utc>,
    dir: store::Handle,
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
impl<S, B> Transform<S, ServiceRequest> for ReqRecord
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

#[derive(Serialize, Deserialize)]
struct ReqHead {
    method: String,
    uri: String,
    version: String,
    headers: BTreeMap<String, String>,
}
struct State {
    store: store::Handle,
    reqhead: ReqHead,
    payload: Rc<RefCell<Vec<u8>>>,
}

#[doc(hidden)]
pub struct Middleware<S> {
    service: S,
    inner: Arc<Inner>,
}
impl<S> Middleware<S> {
    pub fn work_before(&self, req: ServiceRequest) -> actix_web::Result<ServiceRequest> {
        if req.uri().to_string().starts_with("/api") {
            let delta = Utc::now().sub(self.inner.start_time);

            // For instance, incrementing a counter can be safely done by multiple
            // threads using a relaxed fetch_add if you're not using the counter to
            // synchronize any other accesses.
            //
            // It returns the previous value.
            let id = self.inner.count.fetch_add(1, Ordering::Relaxed);

            let (req, payload) = req.into_parts();

            let state = Rc::new(RefCell::new(Vec::<u8>::new()));

            let fwd = payload.scan(state.clone(), |state, x| {
                if let Ok(buf) = &x {
                    state.borrow_mut().extend(buf.to_vec());
                }

                std::future::ready(Some(x))
            });
            let payload: BoxedPayloadStream = Box::pin(fwd);
            req.extensions_mut().insert(State {
                store: self
                    .inner
                    .dir
                    .join(format!("{id:04}_{}", delta.num_milliseconds())),
                payload: state,
                reqhead: ReqHead {
                    method: req.method().to_string(),
                    uri: req.uri().to_string(),
                    version: format!("{:?}", req.version()),
                    headers: req
                        .headers()
                        .iter()
                        .filter_map(|(k, v)| Some((k.to_string(), v.to_str().ok()?.to_string())))
                        .collect(),
                },
            });

            Ok(ServiceRequest::from_parts(req, Payload::from(payload)))
        } else {
            Ok(req)
        }
    }
    pub fn work_after<B: 'static>(
        res: ServiceResponse<B>,
    ) -> actix_web::Result<ServiceResponse<B>> {
        if res.request().uri().to_string().starts_with("/api") {
            if let Some(state) = res.request().extensions().get::<State>() {
                tracing::info!("save request to {}", &state.store);

                state
                    .store
                    .join("head")
                    .serialize_pretty_new_file(&state.reqhead)
                    .context("serialize request header")
                    .map_err(error::ErrorInternalServerError)?;

                // the comsumed part of payload
                let mut payload = state.payload.take();
                let len = payload.len();
                if len > 0 {
                    store::FsStore::save(&mut payload, &state.store.join("payload"))
                        .map_err(error::ErrorInternalServerError)?;
                }
                // if let Ok(s) = String::from_utf8(payload) {
                //     eprintln!("payload (utf8): {s}")
                // } else {
                //     eprintln!("binary payload {len} bytes")
                // }
            } else {
                eprintln!("State not found")
            }
        }
        Ok(res)
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
        let result = self.work_before(req);
        let fut = result.map(|req| self.service.call(req));

        Box::pin(async move {
            let r = fut?.await?;
            let r = Self::work_after(r)?;
            Ok(r)
        })
    }
}
