pub use log::{debug, error, info, trace, warn};

pub use std::io::Error as IOError;
pub use std::io::ErrorKind as IOErrorKind;
pub use std::io::Result as IOResult;

pub use uuid::Uuid;

pub use rocket::request::Form;

pub use serde::{Deserialize, Serialize};

pub use rocket_contrib::templates::Template;

pub use sqlx::Error as SQLError;
pub use sqlx::Result as SQLResult;
pub use sqlx::mysql::MySqlPool;

#[derive(Debug)]
pub enum FError {
    SQLError(SQLError),
    IOError(IOError),
    // DirNotClean(PathBuf),
    // TimeConversionErrorFromSecs(u64),
    // FuseTypeParseError(String),
    // NodeNoNum,
    UuidParseError(String),
    NotImplemented,
    // NoMoreResults,
}

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

pub fn parse_uuid(val: &str) -> FResult<Uuid> {
    match Uuid::parse_str(&val) {
        Ok(v) => Ok(v),
        Err(_) => Err(FError::UuidParseError(val.to_string())),
    }
}