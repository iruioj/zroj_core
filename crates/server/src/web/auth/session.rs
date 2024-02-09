use super::SessionManager;
use actix_session::SessionExt;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error, Error, HttpMessage, Result,
};
use futures::Future;
use std::{
    future::{ready, Ready},
    pin::Pin,
    rc::Rc,
};

pub type SessionID = uuid::Uuid;

// SessionAuth 的内部数据
struct Inner {
    // 是否要求有鉴权信息。如果没有，返回 401 Unauthorized
    require: bool,
    store: SessionManager,
}

/// SessionAuth is a middleware that tries to extracts authentication data
/// and register them into request-local data for future use.
///
/// This middleware relies on the [`actix_session`] crate since it extracts auth. info.
/// from session.
///
/// Usage:
///
/// ```rust
/// # use server::auth::session::SessionAuth;
/// # use server::auth::SessionManager;
/// let session_container = SessionManager::default();
///
/// actix_web::App::new()
///     .wrap(
///         actix_session::SessionMiddleware::builder(
///             actix_session::storage::CookieSessionStore::default(),
///             actix_web::cookie::Key::generate(),
///         )
///         .cookie_secure(false)
///         .cookie_path("/".into())
///         .build(),
///     )
///     .wrap(SessionAuth::require_auth(session_container.clone()));
/// ```
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
            tracing::debug!("session id = {}", id);
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
            tracing::debug!("no session id found");
            // dbg!(req.cookies().unwrap());
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

/// `SessionManager` can be used to manage the authenticating status of users by controling
/// information associated with session id. To enter the login status, one may implement
///
/// ```
/// # use actix_web::web;
/// # use actix_web::HttpResponse;
/// # use server::app::auth::LoginPayload;
/// # use server::auth::session::SessionManager;
/// # use server::auth::AuthInfo;
/// # use server::data::user::UserDB;
/// async fn login(
///     payload: web::Json<LoginPayload>,
///     session_container: web::Data<SessionManager>,
///     user_db: web::Data<UserDB>,
///     session: actix_session::Session,
/// ) -> actix_web::Result<HttpResponse> {
///     use actix_web::cookie::Cookie;
///     tracing::info!("login request: {:?}", payload);
///     let username = payload.username.clone();
///     let user = server::block_it!(user_db.query_by_username(&username))?;
///
///     if !passwd::verify(&user.password_hash, &payload.password_hash) {
///         Err(actix_web::error::ErrorBadRequest("password not correct"))
///     } else {
///         // generate a random session id as session-id
///         let id = server::SessionID::new_v4();
///         tracing::info!("generate new session id {}", id);
///
///         // map session-id to user-id
///         session_container.set(id, AuthInfo { uid: user.id })?;
///
///         // mark the current session with session-id.
///         // We didn't assign a user-id directly for
///         session.insert(server::auth::SESSION_ID_KEY, id)?;
///
///         // add user-id to cookie
///         Ok(HttpResponse::Ok()
///             .cookie(
///                 Cookie::build("username", user.username).path("/").finish()
///             )
///             .body("login success"))
///     }
/// }
/// ```
pub struct SessionManager(pub Arc<RwLock<HashMap<SessionID, AuthInfo>>>);

impl Default for SessionManager {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(HashMap::<SessionID, AuthInfo>::new())))
    }
}

impl SessionManager {
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

pub const SESSION_ID_KEY: &str = "session-id";
