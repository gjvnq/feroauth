pub use log::{debug, error, info, trace, warn};

pub use std::io::Error as IOErrorReal;
pub use std::io::ErrorKind as IOErrorKind;
pub use std::io::Result as IOResult;

pub use std::net::IpAddr;

pub use serde::{Deserialize, Serialize};

pub use sqlx::mysql::MySqlPool;
pub use sqlx::types::Uuid;
pub use sqlx::Error as SQLErrorReal;
pub use sqlx::Result as SQLResult;

pub use chrono::{DateTime, TimeZone, Utc};

pub use actix_web::dev::Body as ActixWebBody;
pub use actix_web::http::header as httpHeader;
pub use actix_web::{web, Either, HttpRequest, HttpResponse, Responder};

pub use argonautica::Error as ArgoErrorReal;

pub use crate::model::*;

use std::panic::Location;

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
pub struct FError {
    file: String,
    line: u32,
    col: u32,
    inner: FErrorInner,
}

#[derive(Debug)]
pub enum FErrorInner {
    #[allow(unused)]
    SerializationError(String),
    #[allow(unused)]
    NotImplemented,
    SQLError(SQLErrorReal),
    IOError(IOErrorReal),
    StaleSession(Uuid),
    UuidParseError(String),
    ArgoError(ArgoErrorReal),
    #[allow(unused)]
    FauxPanic(&'static str, Option<String>),
}

pub use FErrorInner::{
    ArgoError, FauxPanic, IOError, NotImplemented, SQLError, SerializationError, UuidParseError,
};

pub type Transaction<'a> = sqlx::Transaction<'a, sqlx::mysql::MySql>;
pub type FResult<T> = Result<T, FError>;

impl FError {
    #[track_caller]
    pub fn new(inner: FErrorInner) -> Self {
        let loc = Location::caller();
        FError {
            file: loc.file().to_string(),
            line: loc.line(),
            col: loc.column(),
            inner: inner,
        }
    }

    #[track_caller]
    #[allow(unused)]
    pub fn new_faux_panic_1(a: &'static str) -> Self {
        FError::new(FErrorInner::FauxPanic(a, None))
    }

    #[track_caller]
    #[allow(unused)]
    pub fn new_faux_panic_2(a: &'static str, b: Option<String>) -> Self {
        FError::new(FErrorInner::FauxPanic(a, b))
    }

    #[track_caller]
    #[allow(unused)]
    pub fn new_faux_panic_3<T: std::fmt::Debug>(a: &'static str, b: T) -> Self {
        let msg = format!("{:?}", b);
        FError::new_faux_panic_2(a, Some(msg))
    }

    pub fn is_not_found(&self) -> bool {
        match &self.inner {
            SQLError(SQLErrorReal::RowNotFound) => true,
            IOError(err) => match err.kind() {
                std::io::ErrorKind::NotFound => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_unauthorized(&self) -> bool {
        return false;
    }
}

impl std::fmt::Display for FError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_fmt(format_args!("{}:{}:{}", self.file, self.line, self.col))
    }
}

impl actix_web::error::ResponseError for FError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        if self.is_not_found() {
            actix_web::http::StatusCode::NOT_FOUND
        } else if self.is_unauthorized() {
            actix_web::http::StatusCode::UNAUTHORIZED
        } else {
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl std::convert::From<IOErrorReal> for FError {
    #[track_caller]
    fn from(err: IOErrorReal) -> Self {
        let loc = Location::caller();
        FError {
            file: loc.file().to_string(),
            line: loc.line(),
            col: loc.column(),
            inner: IOError(err),
        }
    }
}

impl std::convert::From<SQLErrorReal> for FError {
    #[track_caller]
    fn from(err: SQLErrorReal) -> Self {
        let loc = Location::caller();
        FError {
            file: loc.file().to_string(),
            line: loc.line(),
            col: loc.column(),
            inner: SQLError(err),
        }
    }
}

impl std::convert::From<ArgoErrorReal> for FError {
    #[track_caller]
    fn from(err: ArgoErrorReal) -> Self {
        let loc = Location::caller();
        FError {
            file: loc.file().to_string(),
            line: loc.line(),
            col: loc.column(),
            inner: ArgoError(err),
        }
    }
}

#[track_caller]
pub fn parse_uuid_vec(val: Vec<u8>) -> FResult<Uuid> {
    match Uuid::from_slice(&val) {
        Ok(v) => Ok(v),
        Err(_) => Err(FError::new(UuidParseError(format!("{:?}", val)))),
    }
}
