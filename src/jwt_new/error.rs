use crate::jwt_new::JwtAlgorithm;
use core::panic::Location;
use openssl::error::ErrorStack as SslErrorStackReal;
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
    #[allow(unused)]
    NotImplemented,
    SslErrorStack(SslErrorStackReal),
    AlgHasNoCurveType(JwtAlgorithm),
    UnknownAlg(String),
    UnknownCurve(String),
    UnknownKeyType(String),
    InvalidKey(String),
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

impl std::fmt::Display for JwtError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_fmt(format_args!("{}:{}:{}", self.file, self.line, self.col))
    }
}
