pub use log::{debug, error, info, trace, warn};

pub use std::io::Error as IOError;
pub use std::io::ErrorKind as IOErrorKind;
pub use std::io::Result as IOResult;

use std::sync::{RwLock, Arc};
pub use std::net::IpAddr;

pub use serde::{Deserialize, Serialize};

pub use sqlx::mysql::MySqlPool;
pub use sqlx::types::Uuid;
pub use sqlx::Error as SQLError;
pub use sqlx::Result as SQLResult;

pub use chrono::{DateTime, TimeZone, Utc};

pub use actix_web::dev::Body as ActixWebBody;
pub use actix_web::http::header as httpHeader;
pub use actix_web::{web, Either, HttpRequest, HttpResponse, Responder};

pub use actix_session::{CookieSession, Session};

use jsonwebtoken::encode as jwt_encode;
use jsonwebtoken::errors::Error as JwtError;
pub use jsonwebtoken::Header as JwtHeader;
pub use jsonwebtoken::Algorithm as JwtAlgorithm;

pub use qstring::QString;

pub use argonautica::Error as ArgoError;

pub use crate::model::db::get_tx;
pub use crate::model::{FSession, MinUser};

pub const MIN_NON_EMPTY_STR: usize = 1;

#[derive(Debug, Clone, Default)]
pub struct JwtConfig<'a> {
    pub alg: Option<JwtAlgorithm>,
    pub kid: String,
    pub jku: String,
    pub enc_key: Option<jsonwebtoken::EncodingKey>,
    pub dec_key: Option<jsonwebtoken::DecodingKey<'a>>
}

lazy_static! {
    pub static ref JWT_CONFIG: RwLock<JwtConfig<'static>> = RwLock::new(Default::default());
}

/// Unwraps [`Result`] and returns the inner value or logs the error and panic
#[inline]
#[track_caller]
#[allow(unused)]
pub fn unwrap_or_log<T, E: std::fmt::Debug>(input: Result<T, E>, msg: &str) -> T {
    use std::panic::Location;
    match input {
        Ok(val) => val,
        Err(ref err) => {
            let loc = Location::caller();
            error!(
                "{file}:{line}:{col} {msg}: {err:?}",
                file = loc.file(),
                line = loc.line(),
                col = loc.column(),
                msg = msg,
                err = err
            );
            input.unwrap()
        }
    }
}

#[derive(Debug)]
pub enum InvalidValue {
    OutOfRange(String, usize, usize), // field name, min, max
}

#[derive(Debug)]
pub enum FError {
    SQLError(SQLError),
    IOError(IOError),
    SerializationError(String),
    TemplateError(String, String),
    // InvalidValue(InvalidValue),
    UuidParseError(String),
    ArgoError(ArgoError),
    NotImplemented,
    JwtError(JwtError),
    FauxPanic(&'static str, Option<String>)
}

pub type Transaction<'a> = sqlx::Transaction<'a, sqlx::mysql::MySql>;
pub type FResult<T> = Result<T, FError>;

impl FError {
    pub fn is_not_found(&self) -> bool {
        match self {
            FError::SQLError(SQLError::RowNotFound) => true,
            FError::IOError(err) => match err.kind() {
                std::io::ErrorKind::NotFound => true,
                _ => false,
            },
            _ => false,
        }
    }
}

impl std::fmt::Display for FError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_fmt(format_args!("{:?}", self))
    }
}

impl actix_web::error::ResponseError for FError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        if self.is_not_found() {
            actix_web::http::StatusCode::NOT_FOUND
        } else {
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl std::convert::From<IOError> for FError {
    fn from(err: IOError) -> Self {
        FError::IOError(err)
    }
}

impl std::convert::From<SQLError> for FError {
    fn from(err: SQLError) -> Self {
        FError::SQLError(err)
    }
}

impl std::convert::From<ArgoError> for FError {
    fn from(err: ArgoError) -> Self {
        FError::ArgoError(err)
    }
}

impl std::convert::From<JwtError> for FError {
    fn from(err: JwtError) -> Self {
        FError::JwtError(err)
    }
}

#[allow(unused)]
pub fn parse_uuid_str(val: &str) -> FResult<Uuid> {
    match Uuid::parse_str(&val) {
        Ok(v) => Ok(v),
        Err(_) => Err(FError::UuidParseError(val.to_string())),
    }
}

pub fn parse_uuid_vec(val: Vec<u8>) -> FResult<Uuid> {
    match Uuid::from_slice(&val) {
        Ok(v) => Ok(v),
        Err(_) => Err(FError::UuidParseError(format!("{:?}", val))),
    }
}

// lazy_static! {
//     pub static ref JWT_CONFIG: RwLock<Option<JwtConfig<'static>>> = RwLock::new(None);
// }

// fn get_jwt_config<'a>() -> FResult<JwtConfig> {
//     unsafe {
//         if JWT_CONFIG.is_none() {
//             return Err(FError::NotAvaialble("JWT header not configured".to_string()))
//         }
//         if let Ok(val) = &JWT_CONFIG.unwrap().try_read() {
//             return Ok(val.my_clone())
//         } else {
//             return Err(FError::NotAvaialble("JWT config not readable".to_string()))
//         }
//     }
// }

pub fn set_jwt_config(new: JwtConfig<'static>) -> FResult<()> {
    let mut cfg = match JWT_CONFIG.try_write() {
        Ok(cfg) => cfg,
        _ => return Err(FError::FauxPanic("JWT config not writable", None))
    };
    cfg.alg = new.alg.clone();
    cfg.kid = new.kid.clone();
    cfg.jku = new.jku.clone();
    cfg.enc_key = new.enc_key.clone();
    cfg.dec_key = new.dec_key.clone();

    Ok(())
}

pub fn new_jwt(claims: impl Serialize) -> FResult<String> {
    let cfg = JWT_CONFIG.try_read();
    if cfg.is_err() {
        return Err(FError::FauxPanic("JWT config not readable", None));
    }
    let cfg = cfg.unwrap();
    let alg = match cfg.alg {
        Some(v) => v,
        _ => return Err(FError::FauxPanic("JWT config missing alg", None))
    };
    let enc_key = match &cfg.enc_key {
        Some(v) => v,
        _ => return Err(FError::FauxPanic("JWT config missing enc_key", None))
    };

    let mut header = JwtHeader::new(alg);
    header.kid = Some(cfg.kid.clone());
    header.jku = Some(cfg.jku.clone());
    Ok(jwt_encode(&header, &claims, &enc_key)?)
}