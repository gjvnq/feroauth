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
pub async fn validate_endpoint(data: web::Data<AppState<'_>>, auth: Option<JwtSession>) -> FResult<String> {
    // TODO: transform this into a service and auto refresh token
    // let token = decode_and_refresh_session(&data, &auth).await?;
    // Ok(format!("OK!\n{:?}", token))
    Ok(format!("{:?}", auth))
}

#[derive(Debug)]
pub struct JwtSession {
    inner: Option<MinSession>
}

impl JwtSession {
    fn get_from_ext(extensions: &mut Extensions) -> Self {
        unimplemented!()
        // if let Some(s_impl) = extensions.get::<Rc<RefCell<SessionInner>>>() {
        //     return Session(Rc::clone(&s_impl));
        // }
        // let inner = Rc::new(RefCell::new(SessionInner::default()));
        // extensions.insert(inner.clone());
        // Session(inner)
    }

    pub fn save(&self, req: &mut ServiceRequest) {
        unimplemented!()
        // let session = Session::get_from_ext(&mut *req.extensions_mut());
        // let mut inner = session.0.borrow_mut();
        // inner.state.extend(data);
    }
}

impl FromRequest for JwtSession {
    type Error = AWError;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        ok(JwtSession::get_from_ext(&mut *req.extensions_mut()))
    }
}






#[derive(Debug)]
pub struct JwtAuth(Arc<JwKeyStore>);

impl JwtAuth {
    pub fn new(key_store: JwKeyStore) -> Self {
        JwtAuth(Arc::new(key_store))
    }
}

impl<S, B> Transform<S> for JwtAuth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = AWError>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = AWError;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let key_store = self.0.clone();
        ok(JwtAuthMiddleware { service, key_store })
    }
}

pub struct JwtAuthMiddleware<S> {
    service: S,
    key_store: Arc<JwKeyStore>
}

impl<S, B> Service for JwtAuthMiddleware<S>
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

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        // TODO: change this to allow use of key auto generation (will require RwMutex)
        // let key_store = self.inner.clone();


        println!("Hi from start. You requested: {}", req.path());

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            println!("Hi from response");
            Ok(res)
        })
    }
}