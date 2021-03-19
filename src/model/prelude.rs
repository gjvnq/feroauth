pub use log::{debug, error, info, trace, warn};

pub use std::io::Error as IOError;
pub use std::io::ErrorKind as IOErrorKind;
pub use std::io::Result as IOResult;

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

use jsonwebtoken::errors::Error as JwtError;
use openssl::error::ErrorStack as SslErrorStack;

pub use qstring::QString;

pub use argonautica::Error as ArgoError;

pub use crate::model::db::get_tx;
pub use crate::model::{FSession, MinUser};

pub const MIN_NON_EMPTY_STR: usize = 1;

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
    #[allow(unused)]
    SerializationError(String),
    #[allow(unused)]
    NotImplemented,
    SQLError(SQLError),
    IOError(IOError),
    // InvalidValue(InvalidValue),
    UuidParseError(String),
    ArgoError(ArgoError),
    SslErrorStack(SslErrorStack),
    JwtError(JwtError),
    FauxPanic(&'static str, Option<String>),
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

impl std::convert::From<SslErrorStack> for FError {
    fn from(err: SslErrorStack) -> Self {
        FError::SslErrorStack(err)
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
