pub use crate::jwt_new::error::{JwtError, JwtErrorInner, JwtResult};
pub use openssl::bn::{BigNum, BigNumContext};
pub use openssl::ec::EcGroup;
pub use openssl::ec::EcKey as SslEcKey;
pub use openssl::nid::Nid;
pub use openssl::pkey::PKey;
pub use openssl::pkey::{Private, Public};
pub use openssl::rsa::Rsa as SslRsaKey;
pub use serde::{Deserialize, Serialize};
pub use serde_json::Result as JSResult;
pub use serde_json::Value as JSValue;
pub use std::convert::TryFrom;

#[track_caller]
pub(crate) fn str2bignum(n: &str) -> JwtResult<BigNum> {
    match BigNum::from_dec_str(n) {
        Ok(bn) => Ok(bn),
        Err(err) => Err(JwtError::new(JwtErrorInner::BigNumParseFail(
            n.to_string(),
            format!("{:?}", err),
        ))),
    }
}
