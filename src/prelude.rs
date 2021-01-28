pub use crate::config::Config;

pub use log::{debug, error, info, trace, warn};

pub use std::io::Error as IOError;
pub use std::io::ErrorKind as IOErrorKind;
pub use std::io::Result as IOResult;

pub use uuid::Uuid;

pub use serde::{Deserialize, Serialize};

pub use sqlx::mysql::MySqlPool;
pub use sqlx::Error as SQLError;
pub use sqlx::Executor;
pub use sqlx::Result as SQLResult;

pub use chrono::{DateTime, TimeZone, Utc};

pub use actix_web::Either;
pub use actix_web::web::HttpResponse;
pub use actix_web::dev::Body as ActixWebBody;
pub use actix_web::http::header as httpHeader;

pub use crate::user::User;
pub use crate::db::get_tx;
pub use crate::config::get_config;
pub use crate::templates::exec_html_template;

pub const MIN_NON_EMPTY_STR: usize = 1;

#[derive(Debug)]
pub enum InvalidValue {
    OutOfRange(String, usize, usize), // field name, min, max
}

#[derive(Debug)]
pub enum FError {
    SQLError(SQLError),
    IOError(IOError),
    // DirNotClean(PathBuf),
    // TimeConversionErrorFromSecs(u64),
    // FuseTypeParseError(String),
    // NodeNoNum,
    InvalidValue(InvalidValue),
    UuidParseError(String),
    NotImplemented,
    // NoMoreResults,
}

pub type Transaction<'a> = sqlx::Transaction<'a, sqlx::mysql::MySql>;
pub type FResult<T> = Result<T, FError>;

impl FError {
    // pub fn is_not_found(&self) -> bool {
    //     match self {
    //         FError::SQLError(err) => is_sql_not_found(err),
    //         FError::IOError(err) => match err.kind() {
    //             std::io::ErrorKind::NotFound => true,
    //             _ => false,
    //         },
    //         _ => false,
    //     }
    // }
}

impl std::fmt::Display for FError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { todo!() }
}

impl actix_web::error::ResponseError for FError {
    fn error_response(&self) -> HttpResponse<ActixWebBody> {
        let mut resp = HttpResponse::new(self.status_code());
        resp.headers_mut().insert(
            httpHeader::CONTENT_TYPE,
            httpHeader::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        resp.set_body(ActixWebBody::from("Something went wrong"))
    }
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
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
