use crate::jwt_new::prelude::*;
use crate::jwt_new::*;

#[derive(Debug, Clone)]
pub struct JwtSymetricKeyInner {
    alg: JwtAlgorithm,
    /// [`None`] if non exportable
    private_jwk: Option<JwkRepr>,
    thumbprint_sha256: String,
    exportable: bool,
    k: Vec<u8>,
}

impl JwKeyTraitLowLevel for JwtSymetricKeyInner {
    fn algorithm(&self) -> JwtAlgorithm {
        self.algorithm()
    }
    fn sign_data(&self, data: &[u8]) -> JwtResult<Vec<u8>> {
        self.sign_data(data)
    }
    fn verify_data(&self, data: &[u8], sig: &[u8]) -> JwtResult<()> {
        self.verify_data(data, sig)
    }
    fn is_exportable(&self) -> bool {
        self.is_exportable()
    }
    fn thumbprint_sha256(&self) -> &str {
        &self.thumbprint_sha256
    }
    fn key_type(&self) -> JKeyType {
        self.key_type()
    }
    fn private_key_jwk(&self) -> Option<&JwkRepr> {
        self.private_jwk.as_ref()
    }
}

impl JwtSymetricKeyInner {
    pub fn algorithm(&self) -> JwtAlgorithm {
        self.alg
    }

    pub fn key_type(&self) -> JKeyType {
        JKeyType::JTypeOct
    }

    pub fn is_exportable(&self) -> bool {
        self.exportable
    }

    pub fn thumbprint_sha256(&self) -> &str {
        &self.thumbprint_sha256
    }

    #[allow(unused)]
    fn hash(&self, data: &[u8]) -> JwtResult<Vec<u8>> {
        let mut hasher = self.algorithm().to_hasher()?;
        hasher.update(data)?;
        Ok(hasher.finish()?.to_vec())
    }

    #[allow(unused)]
    fn sign_data(&self, data: &[u8]) -> JwtResult<Vec<u8>> {
        unimplemented!()
    }

    #[allow(unused)]
    fn verify_data(&self, data: &[u8], sig: &[u8]) -> JwtResult<()> {
        unimplemented!()
    }

    // #[allow(unused)]
    // pub(crate) fn generate(crv: ECurve, exportable: bool) -> JwtResult<Self> {
    //     unimplemented!()
    // }

    // #[allow(unused)]
    // pub(crate) fn from_params(
    //     crv: ECurve,
    //     x: &str,
    //     y: &str,
    //     pn: Option<&str>,
    //     exportable: bool,
    // ) -> JwtResult<Self> {
    //     unimplemented!()
    // }
}
