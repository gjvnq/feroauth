use crate::jwt_new::key_ec::JwtEcKeyInner;
use crate::jwt_new::prelude::*;
use crate::jwt_new::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum JwtKey {
    JwtEcKey(JwtEcKeyInner),
    JwtRsaKey(JwtRsaKeyInner),
    JwtSymetricKey(JwtSymetricKeyInner),
}

impl JwtKey {
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

#[derive(Debug, Clone)]
pub struct JwtRsaKeyInner {
    kid: String,
    public: SslRsaKey<Public>,
    public_pem: String,
    public_jwk: JwkRepr,
    private: Option<SslRsaKey<Private>>,
    private_jwk: Option<JwkRepr>,
}
#[derive(Debug, Clone)]
pub struct JwtSymetricKeyInner {
    alg: JwtAlgorithm,
    exportable: bool,
    k: Vec<u8>,
}

pub trait JwtAsymmetricKeyTrait {
    fn algorithm(&self) -> JwtAlgorithm;
    fn public_key_jwk(&self) -> &JwkRepr;
    /// I only added PEM so I could test things easily on jwt.io
    fn public_key_pem(&self) -> &str;
    fn private_key_jwk(&self) -> Option<&JwkRepr>;
    fn has_private(&self) -> bool;
    fn is_exportable(&self) -> bool;
}

pub(crate) trait JwtKeyTraitLowLevel{
    fn sign_data(&self, data: &[u8]) -> JwtResult<Vec<u8>>;
    fn verify_data(&self, data: &[u8], sig: &[u8]) -> JwtResult<()>;
}
