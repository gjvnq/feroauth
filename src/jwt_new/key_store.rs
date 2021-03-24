use crate::jwt_new::prelude::*;
use crate::jwt_new::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
// for now, only JWSs (JSON Web Signatures) are supported
pub struct JwKeyStore {
    /// List of trusted keys organized by `kid`
    keys: HashMap<String, JwWrapedKey>,
    /// List of valid values for the `sub` (subject) param. That param won't be checked iff this field is [`None`].
    sub: Option<Vec<String>>,
    /// List of valid values for the `aud` (audience) param. That param won't be checked iff this field is [`None`].
    aud: Option<Vec<String>>,
    /// List of valid values for the `iss` (issuer) param. That param won't be checked iff this field is [`None`].
    iss: Option<Vec<String>>,
    /// How strict should the verification of deadlines be? (in seconds)
    leeway: usize,
    /// If [`true`], the param `nbf` (not before) won't be checked.
    ignore_nbf: bool,
    /// If [`true`], the param `exp` (not after) won't be checked.
    ignore_exp: bool,
    /// If [`true`], the [`JwKeyStore`] will reject all JWTs without a `kid` param in the headers.
    require_kid: bool,
    /// If [`true`], the JWTs produced will include the `jku` (URL to JWK) param in the header.
    include_jku: bool,
    /// If [`true`], new keys will be generated as needed.
    auto_generate: bool,
    // /// Function to call when a new key is generated
    // callback_new_key: fn(new_key: &JwWrapedKey)
}

#[derive(Debug, Clone)]
pub struct JwWrapedKey {
    key: JwKey,
    // If [`None`], the `kid` will be SHA256 thumbprint (see RFC 7638)
    kid: Option<String>,
    /// Not Before (UNIX Timestamp in seconds)
    nbf: Option<usize>,
    /// Not After (UNIX Timestamp in seconds)
    exp: Option<usize>,
    /// Key URL
    jku: Option<String>,
}

impl JwWrapedKey {
    #[allow(unused)]
    pub fn get_kid(&self) -> &str {
        if let Some(kid) = &self.kid {
            return &kid;
        } else {
            return self.key.thumbprint_sha256();
        }
    }

    #[allow(unused)]
    pub fn is_kid(&self, val: &str) -> bool {
        if let Some(kid) = &self.kid {
            return val == kid;
        } else {
            return val == self.key.thumbprint_sha256();
        }
    }
}

impl JwKeyStore {
    #[allow(unused)]
    pub fn add_key(
        &mut self,
        key: JwKey,
        nbf: Option<usize>,
        exp: Option<usize>,
        jku: Option<String>,
    ) -> JwtResult<()> {
        todo!()
    }

    #[allow(unused)]
    pub fn del_key(&mut self, kid: &str) -> JwtResult<()> {
        todo!()
    }

    #[allow(unused)]
    fn get_key(
        &self,
        kid: Option<&str>,
        alg: Option<JwtAlgorithm>,
        kind: Option<JwkUse>,
    ) -> Option<&JwWrapedKey> {
        todo!()
    }

    #[allow(unused)]
    fn actual_make_token(&self, key: &JwWrapedKey, claims: impl Serialize) -> JwtResult<String> {
        todo!()
    }

    pub fn make_token(
        &self,
        kid: Option<&str>,
        alg: Option<JwtAlgorithm>,
        kind: Option<JwkUse>,
        claims: impl Serialize,
    ) -> JwtResult<String> {
        let key = match self.get_key(kid, alg, kind) {
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
        self.actual_make_token(key, claims)
    }

    /// The same as [`JwKeyStore::make_token`] but will auto generate a new key if needed.
    #[allow(unused)]
    pub fn make_token_autogen(
        &mut self,
        kid: Option<&str>,
        alg: Option<JwtAlgorithm>,
        kind: Option<JwkUse>,
        claims: impl Serialize,
    ) -> JwtResult<String> {
        if self.get_key(kid, alg, kind).is_none() {
            // generate key
        }
        self.make_token(kid, alg, kind, claims)
    }

    #[allow(unused)]
    pub fn parse_token<T>(&self, kid: &str, token: &str) -> JwtResult<JwToken<T>> {
        todo!()
    }
}
