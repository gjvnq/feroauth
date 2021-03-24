use crate::jwt_new::key_asymmetric::JwtAsymmetricKeyInner;
use crate::jwt_new::key_symmetric::JwtSymetricKeyInner;
use crate::jwt_new::prelude::*;
use crate::jwt_new::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum JwKey {
    JwtAsymmetricKey(JwtAsymmetricKeyInner),
    JwtSymetricKey(JwtSymetricKeyInner),
}

impl JwKey {
    #[allow(unused, unused_variables)]
    pub fn generate(alg: JwtAlgorithm) {
        todo!()
    }

    #[allow(unused, unused_variables)]
    pub fn from_jwk(data: &JwkRepr) -> JwtResult<Self> {
        todo!()
    }

    #[allow(unused, unused_variables)]
    pub fn from_jwk_str(data: &str) -> JwtResult<Self> {
        todo!()
    }
}

pub(crate) trait JwKeyTraitLowLevel {
    fn algorithm(&self) -> JwtAlgorithm;
    fn sign_data(&self, data: &[u8]) -> JwtResult<Vec<u8>>;
    fn verify_data(&self, data: &[u8], sig: &[u8]) -> JwtResult<()>;
    fn private_key_jwk(&self) -> Option<&JwkRepr>;
    fn is_exportable(&self) -> bool;
    fn thumbprint_sha256(&self) -> &str;
    fn key_type(&self) -> JKeyType;
}

impl JwKey {
    fn as_trait(&self) -> &dyn JwKeyTraitLowLevel {
        match self {
            JwKey::JwtAsymmetricKey(inner) => inner,
            JwKey::JwtSymetricKey(inner) => inner,
        }
    }

    #[allow(unused)]
    pub fn algorithm(&self) -> JwtAlgorithm {
        self.as_trait().algorithm()
    }

    #[allow(unused)]
    pub fn is_exportable(&self) -> bool {
        self.as_trait().is_exportable()
    }

    pub fn thumbprint_sha256(&self) -> &str {
        self.as_trait().thumbprint_sha256()
    }

    #[allow(unused)]
    pub fn key_type(&self) -> JKeyType {
        self.as_trait().key_type()
    }

    #[allow(unused)]
    /// Returns [`true`] for symmetric keys
    pub fn has_private(&self) -> bool {
        match self {
            JwKey::JwtAsymmetricKey(k) => k.has_private(),
            JwKey::JwtSymetricKey(_) => true,
        }
    }

    #[allow(unused)]
    /// Returns [`None`] for symmetric keys
    pub fn public_key_jwk(&self) -> Option<&JwkRepr> {
        match self {
            JwKey::JwtAsymmetricKey(k) => Some(k.public_key_jwk()),
            JwKey::JwtSymetricKey(_) => None,
        }
    }

    #[allow(unused)]
    /// Returns [`None`] for symmetric keys
    pub fn public_key_pem(&self) -> Option<&str> {
        match self {
            JwKey::JwtAsymmetricKey(k) => Some(k.public_key_pem()),
            JwKey::JwtSymetricKey(_) => None,
        }
    }

    #[allow(unused)]
    /// Returns [`None`] for symmetric keys
    pub fn private_key_jwk(&self) -> Option<&JwkRepr> {
        self.as_trait().private_key_jwk()
    }
}
