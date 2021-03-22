use crate::jwt_new::key_ec::JwtEcKeyInner;
use crate::jwt_new::key_oct::JwtSymetricKeyInner;
use crate::jwt_new::key_rsa::JwtRsaKeyInner;
use crate::jwt_new::prelude::*;
use crate::jwt_new::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum JwKey {
    JwtEcKey(JwtEcKeyInner),
    JwtRsaKey(JwtRsaKeyInner),
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

pub(crate) trait JwtAsymmetricKeyTrait {
    fn algorithm(&self) -> JwtAlgorithm;
    fn public_key_jwk(&self) -> &JwkRepr;
    fn private_key_jwk(&self) -> Option<&JwkRepr>;
    fn has_private(&self) -> bool;
    /// I only added PEM so I could easily test things on jwt.io
    fn public_key_pem(&self) -> &str;
}

pub(crate) trait JwKeyTraitLowLevel {
    fn algorithm(&self) -> JwtAlgorithm;
    fn sign_data(&self, data: &[u8]) -> JwtResult<Vec<u8>>;
    fn verify_data(&self, data: &[u8], sig: &[u8]) -> JwtResult<()>;
    fn is_exportable(&self) -> bool;
    fn thumbprint_sha256(&self) -> &str;
    fn key_type(&self) -> JKeyType;
}

impl JwKey {
    fn as_trait(&self) -> &dyn JwKeyTraitLowLevel {
        match self {
            JwKey::JwtEcKey(inner) => inner,
            JwKey::JwtRsaKey(inner) => inner,
            JwKey::JwtSymetricKey(inner) => inner,
        }
    }

    fn as_trait_asymmetric(&self) -> Option<&dyn JwtAsymmetricKeyTrait> {
        match self {
            JwKey::JwtEcKey(inner) => Some(inner),
            JwKey::JwtRsaKey(inner) => Some(inner),
            JwKey::JwtSymetricKey(_) => None,
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
        self.as_trait_asymmetric()
            .map(|k| k.has_private())
            .unwrap_or(true)
    }

    #[allow(unused)]
    /// Returns [`None`] for symmetric keys
    pub fn public_key_jwk(&self) -> Option<&JwkRepr> {
        self.as_trait_asymmetric().map(|k| k.public_key_jwk())
    }

    #[allow(unused)]
    /// Returns [`None`] for symmetric keys
    pub fn private_key_jwk(&self) -> Option<&JwkRepr> {
        self.as_trait_asymmetric()
            .map(|k| k.private_key_jwk())
            .flatten()
    }

    #[allow(unused)]
    /// Returns [`None`] for symmetric keys
    pub fn public_key_pem(&self) -> Option<&str> {
        self.as_trait_asymmetric().map(|k| k.public_key_pem())
    }
}
