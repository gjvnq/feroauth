pub use crate::model::prelude::*;

pub use actix_web::dev::Body as ActixWebBody;
pub use actix_web::http::header as httpHeader;
pub use actix_web::{web, Either, HttpRequest, HttpResponse, Responder};

pub use actix_session::CookieSession;
pub use actix_session::Session as SSession;

pub use crate::model::{Password, User};
pub use crate::templates::{exec_html_template, basic_html_template, BasicCtx, EmptyCtx};

use tera::Tera;

pub struct AppState {
    pub tmpl: Tera,
    pub db: sqlx::Pool<sqlx::MySql>,
}

pub static mut TMPL: Option<Tera> = None;

pub fn get_ip(req: &HttpRequest) -> (String, String) {
    let ip_addr_real = req.connection_info().realip_remote_addr().unwrap().to_string();
    let ip_addr_peer = req.peer_addr().unwrap().ip().to_string();
    (ip_addr_real, ip_addr_peer)
}

pub fn get_tmpl() -> &'static Tera {
    unsafe { TMPL.as_ref().unwrap() }
}
