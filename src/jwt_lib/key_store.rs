use crate::jwt_lib::prelude::*;
use crate::jwt_lib::*;
use std::collections::HashMap;

const DEFAULT_JWT_ALG: JwtAlgorithm = JwtAlgorithm::ES512;
const DEFAULT_DURATION: i64 = 5 * 60;

#[derive(Debug, Clone, Default)]
// for now, only JWSs (JSON Web Signatures) are supported
pub struct JwKeyStore {
    /// List of trusted keys organized by thumbprint
    keys: HashMap<String, JwWrapedKey>,
    // kid2thumbprint: HashMap<String, String>,
    /// List of valid values for the `sub` (subject) param. That param won't be checked iff this field is [`None`].
    pub sub: Option<Vec<String>>,
    /// List of valid values for the `aud` (audience) param. That param won't be checked iff this field is [`None`].
    pub aud: Option<Vec<String>>,
    /// List of valid values for the `iss` (issuer) param. That param won't be checked iff this field is [`None`].
    pub iss: Option<Vec<String>>,

    pub default_iss: Option<String>,

    pub default_duration: Option<i64>,
    /// How strict should the verification of deadlines be? (in seconds, must be non-negative)
    pub leeway: i64,
    /// If [`true`], the param `nbf` (not before) won't be checked.
    pub ignore_nbf: bool,
    /// If [`true`], the param `exp` (not after) won't be checked.
    pub ignore_exp: bool,
    /// If [`true`], the [`JwKeyStore`] will reject all JWTs without a `kid` param in the headers.
    pub require_kid: bool,
    /// If [`true`], the JWTs produced will include the `jku` (URL to JWK) param in the header.
    pub include_jku: bool,
    /// If [`true`], new keys will be generated as needed.
    pub auto_generate: bool,
    // /// Function to call when a new key is generated
    // callback_new_key: Option<fn(new_key: &JwWrapedKey)>
}

#[derive(Debug, Clone)]
pub struct JwWrapedKey {
    pub key: JwKey,
    // If [`None`], the `kid` will be SHA256 thumbprint (see RFC 7638)
    pub kid: Option<String>,
    /// Not Before (UNIX Timestamp in seconds)
    pub nbf: Option<i64>,
    /// Not After (UNIX Timestamp in seconds)
    pub exp: Option<i64>,
    /// Key URL
    pub jku: Option<String>,
}

impl JwWrapedKey {
    #[allow(unused)]
    pub fn kid_or_thumbprint(&self) -> &str {
        if let Some(kid) = &self.kid {
            return &kid;
        } else {
            return self.key.thumbprint_sha256();
        }
    }

    #[allow(unused)]
    pub fn is_valid(&self, now: i64) -> bool {
        if self.nbf.is_some() && !(self.nbf.unwrap() <= now) {
            return false;
        }
        if self.exp.is_some() && !(self.exp.unwrap() >= now) {
            return false;
        }
        return true;
    }

    #[allow(unused)]
    pub fn check_kid_or_thumbprint(&self, val: &str) -> bool {
        if let Some(kid) = &self.kid {
            return val == kid;
        } else {
            return val == self.key.thumbprint_sha256();
        }
    }
}

impl JwKeyStore {
    pub fn new() -> Self {
        JwKeyStore{..Default::default()}
    }

    #[allow(unused)]
    pub fn add_key(
        &mut self,
        key: JwKey,
        kid: Option<&str>,
        nbf: Option<i64>,
        exp: Option<i64>,
        jku: Option<&str>,
    ) -> JwtResult<String> {
        let val = JwWrapedKey {
            key: key,
            kid: kid.map(|s| s.to_string()),
            nbf: nbf,
            exp: exp,
            jku: jku.map(|s| s.to_string()),
        };
        let thumbprint = val.key.thumbprint_sha256().to_string();
        self.keys.insert(thumbprint.clone(), val);
        Ok(thumbprint)
    }

    #[allow(unused)]
    pub fn del_key(&mut self, thumbprint: &str) -> JwtResult<()> {
        self.keys.remove(thumbprint);
        Ok(())
    }

    #[allow(unused)]
    fn get_key(
        &self,
        kid: Option<&str>,
        alg: Option<JwtAlgorithm>,
        kind: JwkUse,
        now: i64,
    ) -> Option<&JwWrapedKey> {
        for (_, wkey) in self.keys.iter() {
            if wkey.key.kind() != kind {
                continue;
            }
            if kid.is_some() && !wkey.check_kid_or_thumbprint(kid.unwrap()) {
                continue;
            }
            if alg.is_some() && wkey.key.algorithm() != alg.unwrap() {
                continue;
            }
            if wkey.is_valid(now) {
                return Some(wkey);
            }
        }
        return None;
    }

    fn get_key_result(
        &self,
        kid: Option<&str>,
        alg: Option<JwtAlgorithm>,
        kind: JwkUse,
        now: i64,
    ) -> JwtResult<&JwWrapedKey> {
        match self.get_key(kid, alg, kind, now) {
            Some(wkey) => Ok(wkey),
            None => Err(JwtError::new(JwtErrorInner::NoSuchKey {
                kid: kid.map(|s| s.to_string()),
                alg: alg,
                kind: Some(kind),
            })),
        }
    }

    fn decode_and_check_sig(
        &self,
        now: i64,
        token: &str,
    ) -> JwtResult<(JwtHeader, String, String)> {
        let token = token.trim();
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(JwtError::new(JwtErrorInner::MalformedToken(
                "does not have exactly 3 parts",
                token.to_string(),
            )));
        }

        let header = String::from_utf8(base64::decode_config(parts[0], base64::URL_SAFE_NO_PAD)?)?;
        let payload = String::from_utf8(base64::decode_config(parts[1], base64::URL_SAFE_NO_PAD)?)?;
        let header: JwtHeader = serde_json::from_str(&header)?;
        // TODO: consider the case of no key or no kid
        let wkey = self.get_key_result(header.kid.as_deref(), header.alg, JwkUse::Sig, now)?;

        let message = [parts[0], parts[1]].join(".");
        let sig = base64::decode_config(parts[2], base64::URL_SAFE_NO_PAD)?;
        wkey.key.verify_data(message.as_bytes(), &sig)?;

        if header.crit.is_some() {
            return Err(JwtError::new(JwtErrorInner::NotImplemented(
                "`crit` header field".to_string(),
            )));
        }

        Ok((header, wkey.key.thumbprint_sha256().to_string(), payload))
    }

    fn check_basic_claims(&self, now: i64, claims: &str) -> JwtResult<()> {
        let claims: JwtBasicClaims = serde_json::from_str(&claims)?;
        if let Some(exp) = claims.exp {
            if !self.ignore_exp && now < exp + self.leeway {
                return Err(JwtError::new(JwtErrorInner::TokenNotAfter(exp)));
            }
        }
        if let Some(nbf) = claims.nbf {
            if !self.ignore_nbf && nbf < now - self.leeway {
                return Err(JwtError::new(JwtErrorInner::TokenNotBefore(nbf)));
            }
        }
        if let Some(ok_sub) = &self.sub {
            if let Some(sub) = claims.sub {
                if !ok_sub.contains(&sub) {
                    return Err(JwtError::new(JwtErrorInner::InvalidSubject(sub)));
                }
            }
        }
        if let Some(ok_aud) = &self.aud {
            if let Some(aud) = claims.aud {
                if !ok_aud.contains(&aud) {
                    return Err(JwtError::new(JwtErrorInner::InvalidAudience(aud)));
                }
            }
        }
        if let Some(ok_iss) = &self.iss {
            if let Some(iss) = claims.iss {
                if !ok_iss.contains(&iss) {
                    return Err(JwtError::new(JwtErrorInner::InvalidIssuer(iss)));
                }
            }
        }

        Ok(())
    }

    // panics if the key does not exist
    fn actual_make_token(
        &self,
        thumbprint: &str,
        now: i64,
        nbf: JwtTime,
        exp: JwtTime,
        aud: Option<&str>,
        claims: impl Serialize,
    ) -> JwtResult<String> {
        let wkey = self.keys.get(thumbprint).unwrap();
        let mut headers = JwtHeader::new(wkey.key.algorithm(), wkey.kid_or_thumbprint());
        if self.include_jku {
            headers.jku = wkey.jku.clone();
        }
        let headers_b64 = headers.to_base64()?;

        let claims_json_val = serde_json::to_value(claims)?;
        let mut claims_map = match claims_json_val {
            JsonValue::Object(m) => m,
            _ => {
                let mut m = serde_json::Map::new();
                m.insert("data".to_string(), claims_json_val);
                m
            }
        };

        // Ensure token will expire
        let exp = match exp {
            JwtTime::Ignore => JwtTime::Duration(self.default_duration.unwrap_or(DEFAULT_DURATION)),
            _ => exp,
        };

        // Set basic fields: iss, aud, exp, nbf, iat
        if let Some(default_iss) = &self.default_iss {
            claims_map.insert(
                "iss".to_string(),
                JsonValue::String(default_iss.to_string()),
            );
        }
        if let Some(exp) = exp.to_json_value(now) {
            claims_map.insert("exp".to_string(), exp);
        }
        if let Some(nbf) = nbf.to_json_value(now) {
            claims_map.insert("nbf".to_string(), nbf);
        }
        if let Some(aud) = aud {
            claims_map.insert("aud".to_string(), JsonValue::String(aud.to_string()));
        }
        claims_map.insert("iat".to_string(), i64_to_json_value(now));

        let claims_str = serde_json::to_string(&claims_map)?;
        let claims_b64 = base64::encode_config(claims_str, base64::URL_SAFE_NO_PAD);

        let message = headers_b64 + "." + &claims_b64;
        let sig = wkey.key.sign_data(message.as_bytes())?;
        let sig_b64 = base64::encode_config(sig, base64::URL_SAFE_NO_PAD);

        let jwt = format!("{}.{}", message, sig_b64);
        Ok(jwt)
    }

    pub fn make_token(
        &self,
        kid: Option<&str>,
        alg: Option<JwtAlgorithm>,
        kind: Option<JwkUse>,
        aud: Option<&str>,
        claims: impl Serialize,
    ) -> JwtResult<String> {
        let kind_unwrap = kind.unwrap_or(JwkUse::Sig);
        let now = get_time();
        let key = match self.get_key(kid, alg, kind_unwrap, now) {
            Some(key) => key,
            None => {
                let kid = kid.map(|s| s.to_string());
                return Err(JwtError::new(JwtErrorInner::NoSuchKey {
                    kid: kid.clone(),
                    alg: alg,
                    kind: kind,
                }));
            }
        };

        self.actual_make_token(key.kid_or_thumbprint(), now, JwtTime::Fixed(now), JwtTime::Ignore, aud, claims)
    }

    /// The same as [`JwKeyStore::make_token`] but will auto generate a new key if needed.
    #[allow(unused)]
    pub fn make_token_autogen(
        &mut self,
        kid: Option<&str>,
        alg: Option<JwtAlgorithm>,
        kind: Option<JwkUse>,
        aud: Option<&str>,
        claims: impl Serialize,
    ) -> JwtResult<String> {
        let kind = kind.unwrap_or(JwkUse::Sig);
        let now = get_time();
        let key = self.get_key(kid, alg, kind, now);
        if key.is_none() && self.auto_generate {
            let alg = alg.unwrap_or(DEFAULT_JWT_ALG);
            let key = JwKey::generate(alg, true)?;
            let thumbprint = key.thumbprint_sha256();
            let kid = self.add_key(key, None, Some(now), None, None)?;

            self.actual_make_token(&kid, now, JwtTime::Fixed(now), JwtTime::Ignore, aud, claims)
        } else {
            let kid = key.unwrap().kid_or_thumbprint();
            self.actual_make_token(&kid, now, JwtTime::Fixed(now), JwtTime::Ignore, aud, claims)
        }
    }

    #[allow(unused)]
    pub fn parse_token<T: serde::de::DeserializeOwned>(
        &self,
        token: &str,
    ) -> JwtResult<JwToken<T>> {
        let now = get_time();
        let (header, key_thumbprint, claims_str) = self.decode_and_check_sig(now, token)?;
        self.check_basic_claims(now, &claims_str)?;
        let claims = serde_json::from_str(&claims_str)?;

        Ok(JwToken {
            claims: claims,
            kind: JwkUse::Sig,
            header: header,
            key_thumbprint: key_thumbprint,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actual_make_token_1() {
        let mut store = JwKeyStore {
            ..Default::default()
        };
        let ikey = JwtAsymmetricKeyInner::from_params_ec(
            ECurve::ECurveP256,
            "GNJoNtvpSo5G-V4Dvn322vTtGo-TcOPN7PtRhZgeGPU=",
            "UWl0nY5DZGUF8vsjysSNdPVbxG_pcIvDlrmJ2CKpobM=",
            Some("F996fAciHB1q5Hu2ElijVrKNtdNzTADYyo31HPCG8PQ="),
            true,
        )
        .unwrap();
        /*-----BEGIN PUBLIC KEY-----
        MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEGNJoNtvpSo5G+V4Dvn322vTtGo+T
        cOPN7PtRhZgeGPVRaXSdjkNkZQXy+yPKxI109VvEb+lwi8OWuYnYIqmhsw==
        -----END PUBLIC KEY-----*/
        /*-----BEGIN EC PRIVATE KEY-----
        MHcCAQEEIBffenwHIhwdauR7thJYo1ayjbXTc0wA2MqN9RzwhvD0oAoGCCqGSM49
        AwEHoUQDQgAEGNJoNtvpSo5G+V4Dvn322vTtGo+TcOPN7PtRhZgeGPVRaXSdjkNk
        ZQXy+yPKxI109VvEb+lwi8OWuYnYIqmhsw==
        -----END EC PRIVATE KEY-----*/
        let thumbprint = ikey.thumbprint_sha256().to_string();
        let key = JwKey::JwtAsymmetricKey(ikey);
        store.add_key(key, None, None, None, None).unwrap();
        let claims = "hello";
        let jwt = store
            .actual_make_token(
                &thumbprint,
                1,
                JwtTime::Ignore,
                JwtTime::Ignore,
                None,
                claims,
            )
            .unwrap();
        assert_eq!(&jwt[0..154], "eyJ0eXAiOiJKV1QiLCJhbGciOiJFUzI1NiIsImtpZCI6IkVldFBuR184d2FrczhuSzd3alNhRTh4akVWNER6Q2FtZnNvQW1TeVJSbm8ifQ.eyJkYXRhIjoiaGVsbG8iLCJleHAiOjMwMSwiaWF0IjoxfQ.");
    }

    #[test]
    fn test_decode_and_check_sig_1() {
        let mut store = JwKeyStore {
            ..Default::default()
        };
        let ikey = JwtAsymmetricKeyInner::from_params_ec(
            ECurve::ECurveP256,
            "GNJoNtvpSo5G-V4Dvn322vTtGo-TcOPN7PtRhZgeGPU=",
            "UWl0nY5DZGUF8vsjysSNdPVbxG_pcIvDlrmJ2CKpobM=",
            None,
            true,
        )
        .unwrap();
        let key = JwKey::JwtAsymmetricKey(ikey);
        store.add_key(key, None, None, None, None).unwrap();

        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJFUzI1NiIsImtpZCI6IkVldFBuR184d2FrczhuSzd3alNhRTh4akVWNER6Q2FtZnNvQW1TeVJSbm8ifQ.eyJkYXRhIjoiaGVsbG8iLCJleHAiOjMwMSwiaWF0IjoxfQ.bQMMxgRCWnbjyGK5Vsq7CSi8iy09hcHudB-4l3hZNa-b4DjPoDTklwybNZxb18iuInqVybN66GRir_LHtiSXDw";
        let check = store.decode_and_check_sig(0, token);
        println!("{:?}", check);
        assert!(check.is_ok());
        assert_eq!(
            check.unwrap(),
            (
                JwtHeader {
                    typ: Some("JWT".to_string()),
                    alg: Some(JwtAlgorithm::ES256),
                    kid: Some("EetPnG_8waks8nK7wjSaE8xjEV4DzCamfsoAmSyRRno".to_string()),
                    ..Default::default()
                },
                "EetPnG_8waks8nK7wjSaE8xjEV4DzCamfsoAmSyRRno".to_string(),
                "{\"data\":\"hello\",\"exp\":301,\"iat\":1}".to_string()
            )
        );
    }

    #[test]
    fn test_parse_token() {
        let mut store = JwKeyStore {
            ..Default::default()
        };
        let ikey = JwtAsymmetricKeyInner::from_params_ec(
            ECurve::ECurveP256,
            "GNJoNtvpSo5G-V4Dvn322vTtGo-TcOPN7PtRhZgeGPU=",
            "UWl0nY5DZGUF8vsjysSNdPVbxG_pcIvDlrmJ2CKpobM=",
            None,
            true,
        )
        .unwrap();
        let key = JwKey::JwtAsymmetricKey(ikey);
        store.add_key(key, None, None, None, None).unwrap();

        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJFUzI1NiIsImtpZCI6IkVldFBuR184d2FrczhuSzd3alNhRTh4akVWNER6Q2FtZnNvQW1TeVJSbm8ifQ.eyJkYXRhIjoiaGVsbG8iLCJleHAiOjMwMSwiaWF0IjoxfQ.bQMMxgRCWnbjyGK5Vsq7CSi8iy09hcHudB-4l3hZNa-b4DjPoDTklwybNZxb18iuInqVybN66GRir_LHtiSXDw";
        let check = store.parse_token::<JwtBasicClaims>(token).unwrap();
        println!("{:?}", check);
        assert_eq!(
            check.claims,
            JwtBasicClaims {
                sub: None,
                iss: None,
                aud: None,
                exp: Some(301),
                nbf: None
            }
        );
        assert_eq!(check.kind, JwkUse::Sig);
        assert_eq!(
            check.header,
            JwtHeader {
                typ: Some("JWT".to_string()),
                crit: None,
                alg: Some(JwtAlgorithm::ES256),
                cty: None,
                jku: None,
                kid: Some("EetPnG_8waks8nK7wjSaE8xjEV4DzCamfsoAmSyRRno".to_string()),
                x5u: None,
                x5t: None
            }
        );
        assert_eq!(
            check.key_thumbprint,
            "EetPnG_8waks8nK7wjSaE8xjEV4DzCamfsoAmSyRRno".to_string()
        );
    }
}
