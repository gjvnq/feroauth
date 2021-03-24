use crate::jwt_lib::JwkUse;
use crate::jwt_lib::JwtAlgorithm;
use base64::DecodeError as B64DecodeErrorReal;
use core::panic::Location;
use openssl::error::ErrorStack as SslErrorStackReal;
use serde_json::error::Error as SerdeJsonErrorReal;
use std::string::FromUtf8Error;

pub type JwtResult<T> = Result<T, JwtError>;

#[derive(Debug)]
pub struct JwtError {
    file: String,
    line: u32,
    col: u32,
    inner: JwtErrorInner,
}

#[derive(Debug)]
pub enum JwtErrorInner {
    NotImplemented(String),
    SslErrorStack(SslErrorStackReal),
    AlgHasNoCurveType(JwtAlgorithm),
    UnknownAlg(String),
    UnknownCurve(String),
    UnknownKeyType(String),
    InvalidKey(String),
    NoPrivateKeyForPubKey(String),
    TokenNotAfter(i64),
    TokenNotBefore(i64),
    InvalidSubject(String),
    InvalidAudience(String),
    InvalidIssuer(String),
    InvalidSignature {
        kid: String,
        data: String,
        sig: String,
    },
    UnknownKeyUse(String),
    NoSuchKey {
        kid: Option<String>,
        alg: Option<JwtAlgorithm>,
        kind: Option<JwkUse>,
    },
    B64ParseError(B64DecodeErrorReal),
    SerdeJson(SerdeJsonErrorReal),
    MalformedToken(&'static str, String),
    BigNumParseFail(String, String),
    Utf8Error(FromUtf8Error),
    Panic(&'static str, Option<String>),
}

impl JwtError {
    #[track_caller]
    pub fn new(inner: JwtErrorInner) -> Self {
        let loc = Location::caller();
        JwtError {
            file: loc.file().to_string(),
            line: loc.line(),
            col: loc.column(),
            inner: inner,
        }
    }

    #[track_caller]
    pub fn new_panic_1(a: &'static str) -> Self {
        JwtError::new(JwtErrorInner::Panic(a, None))
    }

    #[track_caller]
    pub fn new_panic_2(a: &'static str, b: Option<String>) -> Self {
        JwtError::new(JwtErrorInner::Panic(a, b))
    }

    #[track_caller]
    pub fn new_panic_3<T: std::fmt::Debug>(a: &'static str, b: T) -> Self {
        let msg = format!("{:?}", b);
        JwtError::new_panic_2(a, Some(msg))
    }
}

impl std::convert::From<SslErrorStackReal> for JwtError {
    #[track_caller]
    fn from(err: SslErrorStackReal) -> Self {
        let loc = Location::caller();
        JwtError {
            file: loc.file().to_string(),
            line: loc.line(),
            col: loc.column(),
            inner: JwtErrorInner::SslErrorStack(err),
        }
    }
}

impl std::convert::From<FromUtf8Error> for JwtError {
    #[track_caller]
    fn from(err: FromUtf8Error) -> Self {
        let loc = Location::caller();
        JwtError {
            file: loc.file().to_string(),
            line: loc.line(),
            col: loc.column(),
            inner: JwtErrorInner::Utf8Error(err),
        }
    }
}

impl std::convert::From<SerdeJsonErrorReal> for JwtError {
    #[track_caller]
    fn from(err: SerdeJsonErrorReal) -> Self {
        let loc = Location::caller();
        JwtError {
            file: loc.file().to_string(),
            line: loc.line(),
            col: loc.column(),
            inner: JwtErrorInner::SerdeJson(err),
        }
    }
}

impl std::convert::From<B64DecodeErrorReal> for JwtError {
    #[track_caller]
    fn from(err: B64DecodeErrorReal) -> Self {
        let loc = Location::caller();
        JwtError {
            file: loc.file().to_string(),
            line: loc.line(),
            col: loc.column(),
            inner: JwtErrorInner::B64ParseError(err),
        }
    }
}

impl std::fmt::Display for JwtError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_fmt(format_args!("{}:{}:{}", self.file, self.line, self.col))
    }
}
