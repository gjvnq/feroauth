pub use crate::model::prelude::*;

pub use actix_web::dev::Body as ActixWebBody;
pub use actix_web::http::header as httpHeader;
pub use actix_web::{web, Either, HttpRequest, HttpResponse, Responder};

pub use actix_session::{CookieSession, Session};

pub use crate::model::{Password, User};

use tera::Tera;
pub struct AppState {
    pub tmpl: Tera,
    pub db: sqlx::Pool<sqlx::MySql>,
}
