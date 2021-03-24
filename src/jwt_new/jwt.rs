use crate::jwt_new::prelude::*;
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
    pub kid: String,
    pub kind: JwkUse,
    pub header: JwtHeader,
    pub claims: T,
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
    pub exp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub enum JwtTime {
    Fixed(u64),
    Duration(i64),
    Ignore,
}

impl JwtTime {
    fn delta_plus_now(delta: i64, now: u64) -> u64 {
        let now: i64 = now.try_into().expect("time should be i64 representable");
        let ans = now + delta;
        ans.try_into().expect("time should be u64 representable")
    }

    #[allow(unused)]
    pub fn to_fixed(&self, now: u64) -> Self {
        match self {
            JwtTime::Duration(delta) => JwtTime::Fixed(JwtTime::delta_plus_now(*delta, now)),
            _ => *self,
        }
    }

    pub fn to_u64(&self, now: u64) -> Option<u64> {
        match self {
            JwtTime::Duration(delta) => Some(JwtTime::delta_plus_now(*delta, now)),
            JwtTime::Fixed(ans) => Some(*ans),
            JwtTime::Ignore => None,
        }
    }

    pub(crate) fn to_json_value(&self, now: u64) -> Option<JsonValue> {
        self.to_u64(now).map(|num| u64_to_json_value(num))
    }
}
