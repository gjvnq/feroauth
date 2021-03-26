use actix_web::http::{HeaderName, HeaderValue};
use std::rc::Rc;
use actix_web::dev::Extensions;
use crate::jwt_lib::*;
use std::sync::Arc;
use crate::prelude::*;
use std::task::{Context, Poll};
use futures_util::future::Future;
use std::pin::Pin;
use futures_util::future::ok;
use futures_util::future::Ready;
use actix_web::FromRequest;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, Payload};
use actix_web::Error as AWError;

// #[post("/keys")]
// async fn keys_endpoint(data: web::Data<AppState<'_>>, _req: HttpRequest) -> FResult<String> {
//     let ans = format!("[{}]", data.jwt.public_key_jwk());
//     Ok(ans)
// }

#[get("/validate")]
pub async fn validate_endpoint(_data: web::Data<AppState>, auth: Option<JwtBearer<MinSession>>) -> FResult<String> {
    // TODO: transform this into a service and auto refresh token
    // let token = decode_and_refresh_session(&data, &auth).await?;
    // Ok(format!("OK!\n{:?}", token))
    Ok(format!("{:?}", auth))
}

#[derive(Debug)]
pub struct JwtBearer<T: std::fmt::Debug> {
    inner: Rc<crate::jwt_lib::JwtResult<crate::jwt_lib::JwToken<T>>>
}
// pub struct JwtBearer<T> {
//     inner: JwtResult<T>
// }

impl<T: 'static +  std::fmt::Debug> JwtBearer<T> {
    fn clone(&self) -> Self {
        return JwtBearer{ inner: self.inner.clone() };
    }

    fn get_from_ext(extensions: &mut Extensions) -> Self {
        use crate::jwt_lib::JwtError;

        println!("{:?}", extensions);
        // println!("{:?}", extensions.get::<JwtBearer<T>>());
        if let Some(token) = extensions.get::<JwtBearer<T>>() {
            // println!("Got: {:?}", token);
            let copy = token.clone();
            return copy;
        }
        
        return JwtBearer{inner: Rc::new(Err(JwtError::new(JwtErrorInner::TokenNotPresent)))};
    }

    // pub fn save(&self, req: &mut ServiceRequest) {
    //     unimplemented!()
    //     // let session = Session::get_from_ext(&mut *req.extensions_mut());
    //     // let mut inner = session.0.borrow_mut();
    //     // inner.state.extend(data);
    // }
}

impl<T: 'static +  std::fmt::Debug> FromRequest for JwtBearer<T> {
    type Error = AWError;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ok(JwtBearer::get_from_ext(&mut *req.extensions_mut()))
    }
}

use std::marker::PhantomData;

#[derive(Debug)]
pub struct JwtAuth<T>(Arc<JwKeyStore>, PhantomData<T>);

impl<T> JwtAuth<T> {
    pub fn new(key_store: JwKeyStore) -> Self {
        JwtAuth(Arc::new(key_store), PhantomData)
    }
}

impl<S, B, T> Transform<S> for JwtAuth<T>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = AWError>,
    T: 'static + std::fmt::Debug + serde::de::DeserializeOwned,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = AWError;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S,T>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let key_store = self.0.clone();
        ok(JwtAuthMiddleware { service, key_store, _marker: PhantomData })
    }
}

pub struct JwtAuthMiddleware<S,T> {
    service: S,
    key_store: Arc<JwKeyStore>,
    _marker: PhantomData<T>
}

impl<S,T> JwtAuthMiddleware<S,T>
where T: 'static + std::fmt::Debug + serde::de::DeserializeOwned {
    fn before_request(&mut self, req: &mut ServiceRequest) {
        if let Some(auth_header) = req.headers().get("Authorization") {
            // Get JWT propper
            let auth_header = match auth_header.to_str() {
                Ok(v) => v,
                Err(_) => {error!("Authorization header must be ASCII only: {:?}", auth_header); return}
            };
            let auth_header_parts: Vec<&str> = auth_header.split(' ').collect();
            if auth_header_parts.len() != 2 {
                error!("Authorization header must have exactly 2 parts, got {}: {:?}", auth_header_parts.len(), auth_header_parts);
                return
            }
            if auth_header_parts[0] != "Bearer" {
                error!("Authorization header must begin with 'Bearer', got {:?}", auth_header_parts[0]);
                return
            }
            println!("{}", auth_header_parts[1]);

            // Authenticate and decode it
            let inner = Rc::new(self.key_store.parse_token::<T>(auth_header_parts[1]));
            req.head_mut().extensions_mut().insert(JwtBearer{inner});
        }
    }
}

impl<S, B, T> Service for JwtAuthMiddleware<S,T>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = AWError>,
    T: 'static + std::fmt::Debug + serde::de::DeserializeOwned,
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
        // let key_store = self.key_store.clone();

        println!("Hi from start. You requested: {}", req.path());
        self.before_request(&mut req);


        println!("Hi from start. You requested: {:?}", req);

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            use std::str::FromStr;

            res.headers_mut().insert(HeaderName::from_str("Set-Authorization").unwrap(), HeaderValue::from_str("hi").unwrap());

            println!("Hi from response");
            Ok(res)
        })
    }
}