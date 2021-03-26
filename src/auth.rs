use std::cell::RefCell;
use std::rc::Rc;
use actix_web::http::{HeaderMap, HeaderName, HeaderValue};



use std::sync::Arc;
use crate::prelude::*;
use std::task::{Context, Poll};
use futures_util::future::Future;
use std::pin::Pin;
use futures_util::future::ok;
use futures_util::future::Ready;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error as AWError;

// #[post("/keys")]
// async fn keys_endpoint(data: web::Data<AppState<'_>>, _req: HttpRequest) -> FResult<String> {
//     let ans = format!("[{}]", data.jwt.public_key_jwk());
//     Ok(ans)
// }

#[get("/validate")]
pub async fn validate_endpoint(_data: web::Data<AppState>, auth: Option<FullSession>) -> FResult<String> {
    // TODO: transform this into a service and auto refresh token
    // let token = decode_and_refresh_session(&data, &auth).await?;
    // Ok(format!("OK!\n{:?}", token))
    Ok(format!("{:?}", auth))
}

#[derive(Debug)]
pub struct SessionAuth(Arc<sqlx::Pool<sqlx::MySql>>, String);

impl SessionAuth {
    pub fn new(cookie_name: &str, db_pool: Arc<sqlx::Pool<sqlx::MySql>>) -> Self {
        SessionAuth(db_pool, cookie_name.to_string())
    }
}

impl<S: 'static, B> Transform<S> for SessionAuth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = AWError>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = AWError;
    type InitError = ();
    type Transform = SessionAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let db_pool = self.0.clone();
        let cookie_name = self.1.clone();
        ok(SessionAuthMiddleware { service: Rc::new(RefCell::new(service)), db_pool, cookie_name })
    }
}

#[derive(Debug)]
pub struct SessionAuthMiddleware<S> {
    service: Rc<RefCell<S>>,
    cookie_name: String,
    db_pool: Arc<sqlx::Pool<sqlx::MySql>>,
}

impl<S> SessionAuthMiddleware<S> {
    fn clone(&self) -> Self {
        SessionAuthMiddleware {
            service: self.service.clone(),
            cookie_name: self.cookie_name.clone(),
            db_pool: self.db_pool.clone(),
        }
    }

    fn get_session_uuid(&self, headers: &HeaderMap) -> Option<Uuid> {
        use actix_web::http::header::COOKIE;
        use cookie::Cookie;

        for cookie in headers.get_all(COOKIE) {
            let cookie = match cookie.to_str() {
                Ok(v) => v,
                Err(err) => {
                    warn!("Failed to parse cookie {:?}: {:?}", cookie, err);
                    continue;
                }
            };
            let cookie = match Cookie::parse(cookie) {
                Ok(v) => v,
                Err(err) => {
                    warn!("Failed to parse cookie {:?}: {:?}", cookie, err);
                    continue;
                }
            };
            if cookie.name() == self.cookie_name {
                use std::str::FromStr;

                let val = cookie.value();
                let val = match Uuid::from_str(val) {
                    Ok(v) => v,
                    Err(err) => {
                        warn!("Failed to parse UUID {:?}: {:?}", val, err);
                        continue;
                    }
                };
                return Some(val);
            }
        }
        None
    }

    async fn before_request(&mut self, req: &mut ServiceRequest) {
        let session_uuid = match self.get_session_uuid(req.head().headers()) {
            Some(v) => v,
            None => return
        };
        println!("{:?}", session_uuid);
        let session = match FullSession::safe_load_by_uuid(session_uuid, self.db_pool.clone()).await {
            Ok(v) => v,
            Err(err) => {
                if !err.is_not_found() {
                    warn!("Failed to get session {}: {:?}", session_uuid, err);
                }
                return
            }
        };
        req.head().extensions_mut().insert(session);
    }

    fn after_response<B>(&self, res: &mut ServiceResponse<B>) {
        if let Some(session) = res.request().head().extensions().get::<FullSession>() {
            println!("Got: {:?}", session);
            // set cookie
        }
    }
}

impl<S: 'static, B> Service for SessionAuthMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = AWError>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = AWError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let mut self2 = self.clone();

        Box::pin(async move {
            self2.before_request(&mut req).await;
            let fut = self2.service.call(req);
            let mut res = fut.await?;
            self2.after_response(&mut res);
            Ok(res)
        })
    }
}