use crate::jwt_new::prelude::*;

#[derive(Debug,Clone)]
pub struct JwToken<T> {
    pub header: JwtHeader,
    pub claims: T
}

#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct JwtHeader {
    pub typ: Option<String>,
    pub alg: Option<JwtAlgorithm>,
    pub cty: Option<String>,
    pub jku: Option<String>,
    pub kid: Option<String>,
    pub x5u: Option<String>,
    pub x5t: Option<String>,
}

impl JwtHeader {
    pub fn new(alg: JwtAlgorithm, kid: &str) -> JwtHeader {
        JwtHeader{
            typ: Some("JWT".to_string()),
            alg: Some(alg),
            kid: Some(kid.to_string()),
            ..Default::default()
        }
    }

    pub fn to_base64(&self) -> JwtResult<String> {
        let json = serde_json::to_string(self)?;
        Ok(base64::encode(json))
    }
}