pub use crate::jwt_new::error::{JwtError, JwtErrorInner, JwtResult};
pub use crate::jwt_new::jwk::{JwkUse, JwtAlgorithm};
pub use crate::jwt_new::jwt::JwToken;
use openssl::bn::BigNumRef;
pub use openssl::bn::{BigNum, BigNumContext};
pub use openssl::ec::EcGroup;
pub use openssl::ec::EcKey as SslEcKey;
pub use openssl::hash::{hash as openssl_hash, Hasher as SslHasher, MessageDigest};
pub use openssl::nid::Nid;
pub use openssl::pkey::PKey;
pub use openssl::pkey::{Private, Public};
pub use openssl::rsa::Rsa as SslRsaKey;
pub use openssl::sign::{Signer as SslSigner, Verifier as SslVerifier};
pub use serde::de::DeserializeOwned;
pub use serde::{Deserialize, Serialize};
pub use serde_json::Result as JSResult;
pub use serde_json::Value as JSValue;
pub use std::convert::TryFrom;

#[track_caller]
pub(crate) fn dec_to_bn(n: &str) -> JwtResult<BigNum> {
    match BigNum::from_dec_str(n) {
        Ok(bn) => Ok(bn),
        Err(err) => Err(JwtError::new(JwtErrorInner::BigNumParseFail(
            n.to_string(),
            format!("{:?}", err),
        ))),
    }
}

pub(crate) fn bn_to_b64(num: &BigNumRef) -> String {
    let num = num.to_vec();
    base64::encode_config(num, base64::URL_SAFE_NO_PAD)
}

#[track_caller]
pub(crate) fn b64_to_bn(num_str: &str) -> JwtResult<BigNum> {
    let num_vec = base64::decode_config(num_str, base64::URL_SAFE_NO_PAD)?;
    Ok(BigNum::from_slice(&num_vec)?)
}

pub(crate) fn option_bn_to_b64(num: Option<&BigNumRef>) -> Option<String> {
    num.map(|n| bn_to_b64(n))
}

#[track_caller]
pub(crate) fn option_b64_to_bn(num_str: Option<&str>) -> JwtResult<Option<BigNum>> {
    match num_str {
        Some(num_str) => b64_to_bn(num_str).map(|bn| Some(bn)),
        None => Ok(None),
    }
}
