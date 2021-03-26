pub use crate::model::prelude::*;

pub use actix_web::dev::Body as ActixWebBody;
pub use actix_web::http::header as httpHeader;
pub use actix_web::{web, Either, HttpRequest, HttpResponse, Responder};

pub use actix_session::CookieSession;
pub use actix_session::Session as SSession;
pub use actix_web_httpauth::extractors::bearer::BearerAuth;

pub use crate::model::{Password, User};

pub use jsonwebtoken::Algorithm as JwtAlgorithm;

pub struct AppState {
    pub db: sqlx::Pool<sqlx::MySql>,
    pub jwt: crate::jwt_lib::JwKeyStore,
}

pub fn get_ip(req: &HttpRequest) -> (String, String) {
    let ip_addr_real = req
        .connection_info()
        .realip_remote_addr()
        .unwrap()
        .to_string();
    let ip_addr_peer = req.peer_addr().unwrap().ip().to_string();
    (ip_addr_real, ip_addr_peer)
}

// pub async fn decode_and_refresh_session(
//     data: &AppState,
//     auth: &BearerAuth,
// ) -> FResult<MinSession> {
//     let token = data.jwt.decode_session(auth)?;
//     token.refresh(&data.db).await?;
//     Ok(token)
// }
