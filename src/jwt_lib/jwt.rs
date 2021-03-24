use crate::jwt_lib::prelude::*;
use std::convert::TryInto;

// #[derive(Debug,Clone,Copy)]
// pub enum JwtKind {
//     Sig,
//     Enc,
//     // First sign, then encrypt
//     // EncSig
// }
#[derive(Debug, Clone)]
pub struct JwToken<T> {
    pub claims: T,
    pub kind: JwkUse,
    pub header: JwtHeader,
    pub key_thumbprint: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct JwtHeader {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crit: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alg: Option<JwtAlgorithm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jku: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x5u: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x5t: Option<String>,
}

impl JwtHeader {
    pub fn new(alg: JwtAlgorithm, kid: &str) -> JwtHeader {
        JwtHeader {
            typ: Some("JWT".to_string()),
            alg: Some(alg),
            kid: Some(kid.to_string()),
            ..Default::default()
        }
    }

    pub fn to_base64(&self) -> JwtResult<String> {
        let json = serde_json::to_string(self)?;
        Ok(base64::encode_config(json, base64::URL_SAFE_NO_PAD))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct JwtBasicClaims {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
}

#[derive(Debug, Clone, Copy)]
pub enum JwtTime {
    Fixed(i64),
    Duration(i64),
    Ignore,
}

impl JwtTime {
    fn delta_plus_now(delta: i64, now: i64) -> i64 {
        let now: i64 = now.try_into().expect("time should be i64 representable");
        let ans = now + delta;
        ans.try_into().expect("time should be i64 representable")
    }

    #[allow(unused)]
    pub fn to_fixed(&self, now: i64) -> Self {
        match self {
            JwtTime::Duration(delta) => JwtTime::Fixed(JwtTime::delta_plus_now(*delta, now)),
            _ => *self,
        }
    }

    pub fn to_i64(&self, now: i64) -> Option<i64> {
        match self {
            JwtTime::Duration(delta) => Some(JwtTime::delta_plus_now(*delta, now)),
            JwtTime::Fixed(ans) => Some(*ans),
            JwtTime::Ignore => None,
        }
    }

    pub(crate) fn to_json_value(&self, now: i64) -> Option<JsonValue> {
        self.to_i64(now).map(|num| i64_to_json_value(num))
    }
}
