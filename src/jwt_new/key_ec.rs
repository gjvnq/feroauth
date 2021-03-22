use crate::jwt_new::prelude::*;
use crate::jwt_new::*;

#[derive(Debug, Clone)]
pub struct JwtEcKeyInner {
    kid: String,
    public: PKey<Public>,
    public_pem: String,
    public_jwk: JwkRepr,
    private: Option<PKey<Private>>,
    private_jwk: Option<JwkRepr>,
}

impl JwtKeyTraitLowLevel for JwtEcKeyInner {
    fn sign_data(&self, data: &[u8]) -> JwtResult<Vec<u8>> {
        self.sign_data(data)
    }
    fn verify_data(&self, data: &[u8], sig: &[u8]) -> JwtResult<()> {
        self.verify_data(data, sig)
    }
}

impl JwtAsymmetricKeyTrait for JwtEcKeyInner {
    fn algorithm(&self) -> JwtAlgorithm {
        self.algorithm()
    }
    fn public_key_jwk(&self) -> &JwkRepr {
        self.public_key_jwk()
    }
    fn public_key_pem(&self) -> &str {
        self.public_key_pem()
    }
    fn private_key_jwk(&self) -> Option<&JwkRepr> {
        self.private_key_jwk()
    }
    fn has_private(&self) -> bool {
        self.has_private()
    }
    fn is_exportable(&self) -> bool {
        self.is_exportable()
    }
}

impl JwtEcKeyInner {
    pub fn algorithm(&self) -> JwtAlgorithm {
        self.public_jwk.crv.unwrap().to_alg()
    }

    pub fn public_key_jwk(&self) -> &JwkRepr {
        &self.public_jwk
    }

    pub fn public_key_pem(&self) -> &str {
        &self.public_pem
    }

    pub fn private_key_jwk(&self) -> Option<&JwkRepr> {
        // if the key is non exportable, this field would be blank
        self.private_jwk.as_ref()
    }

    pub fn has_private(&self) -> bool {
        self.private.is_some()
    }

    pub fn is_exportable(&self) -> bool {
        self.private_jwk.is_some()
    }

    fn hash(&self, data: &[u8]) -> JwtResult<Vec<u8>> {
        let mut hasher = self.algorithm().to_hasher()?;
        hasher.update(data)?;
        Ok(hasher.finish()?.to_vec())
    }

    fn sign_data(&self, data: &[u8]) -> JwtResult<Vec<u8>> {
        let sk = match &self.private {
            Some(key) => key,
            None => return Err(JwtError::new(JwtErrorInner::NoPrivateKeyForPubKey(self.kid.clone())))
        };
        let mut signer = SslSigner::new(self.algorithm().to_md(), sk)?;
        let ans = signer.sign_oneshot_to_vec(data)?;
        Ok(ans)
    }

    fn verify_data(&self, data: &[u8], sig: &[u8]) -> JwtResult<()> {
        let pk = &self.public;
        let mut verifier = SslVerifier::new(self.algorithm().to_md(), pk)?;
        match verifier.verify_oneshot(sig, data)? {
            true => Ok(()),
            false => {
                // pretty print stuff and return the error
                let hash = self.hash(data).map(|v| hex::encode(v)).unwrap_or(base64::encode(data));
                let sig = base64::encode(sig);
                Err(JwtError::new(JwtErrorInner::InvalidSignature(self.kid.clone(), hash, sig)))
            }
        }
        
    }

    fn get_crv_x_y(key: &SslEcKey<Public>) -> JwtResult<(ECurve, String, String)> {
        // Get key parameters
        let ec_group = key.group();
        let crv = match ec_group.curve_name() {
            Some(v) => v,
            None => {
                return Err(JwtError::new(JwtErrorInner::InvalidKey(
                    "EcGroup.curve_name() returned None".to_string(),
                )))
            }
        };

        let crv = ECurve::try_from(crv)?;
        let point = key.public_key();
        let mut bigctx = BigNumContext::new()?;
        let mut x = BigNum::new()?;
        let mut y = BigNum::new()?;
        point.affine_coordinates_gfp(ec_group, &mut x, &mut y, &mut bigctx)?;
        Ok((crv, x.to_string(), y.to_string()))
    }

    fn make_jwk_public(key: &SslEcKey<Public>) -> JwtResult<JwkRepr> {
        let (crv, x, y) = JwtEcKeyInner::get_crv_x_y(key)?;
        Ok(JwkRepr {
            kty: Some(JKeyType::JTypeEc),
            crv: Some(crv),
            x: Some(x),
            y: Some(y),
            ..Default::default()
        })
    }

    fn make_jwk_private(ans: &JwkRepr, key: &SslEcKey<Private>) -> JwkRepr {
        let mut ans = ans.clone();
        ans.p = Some(key.private_key().to_string());
        ans
    }

    pub(crate) fn generate(crv: ECurve, exportable: bool) -> JwtResult<Self> {
        let ecg_nid = crv.to_nid();
        let ecg = EcGroup::from_curve_name(ecg_nid)?;
        let pri_key = SslEcKey::generate(&ecg)?;
        let pub_key = SslEcKey::from_public_key(&ecg, &pri_key.public_key())?;

        if let Err(err) = pub_key.check_key() {
            return Err(JwtError::new(JwtErrorInner::InvalidKey(format!(
                "crv={} err={:?}",
                crv, err
            ))));
        }
        if let Err(err) = pri_key.check_key() {
            return Err(JwtError::new(JwtErrorInner::InvalidKey(format!(
                "crv={} err={:?}",
                crv, err
            ))));
        }

        let public_jwk = JwtEcKeyInner::make_jwk_public(&pub_key)?;
        let private_jwk = match exportable {
            true => Some(JwtEcKeyInner::make_jwk_private(&public_jwk, &pri_key)),
            false => None,
        };

        let public_pem = String::from_utf8(pub_key.public_key_to_pem()?)?;

        Ok(JwtEcKeyInner {
            kid: public_jwk.thumbprint_sha256(),
            public: PKey::from_ec_key(pub_key)?,
            public_jwk,
            public_pem,
            private: Some(PKey::from_ec_key(pri_key)?),
            private_jwk,
        })
    }

    #[allow(unused)]
    pub(crate) fn from_params(
        crv: ECurve,
        x: &str,
        y: &str,
        pn: Option<&str>,
        exportable: bool,
    ) -> JwtResult<Self> {
        let x = str2bignum(x)?;
        let y = str2bignum(y)?;

        let ecg_nid = crv.to_nid();
        let ecg = EcGroup::from_curve_name(ecg_nid)?;
        let pub_key = SslEcKey::from_public_key_affine_coordinates(&ecg, &x, &y)?;

        if let Err(err) = pub_key.check_key() {
            return Err(JwtError::new(JwtErrorInner::InvalidKey(format!(
                "crv={} x={} y={} err={:?}",
                crv, x, y, err
            ))));
        }

        let public_jwk = JwtEcKeyInner::make_jwk_public(&pub_key)?;
        let public_pem = String::from_utf8(pub_key.public_key_to_pem()?)?;

        let mut pri_key = None;
        let mut private_jwk = None;
        if let Some(pn) = pn {
            // Add private key info if provided by user
            let pn = str2bignum(pn)?;
            let point = pub_key.public_key();
            let pri_key_raw = SslEcKey::from_private_components(&ecg, &pn, &point)?;
            if let Err(err) = pri_key_raw.check_key() {
                return Err(JwtError::new(JwtErrorInner::InvalidKey(format!(
                    "crv={} x={} y={} p=Some(...) err={:?}",
                    crv, x, y, err
                ))));
            }
            if exportable {
                private_jwk = Some(JwtEcKeyInner::make_jwk_private(&public_jwk, &pri_key_raw));
            }
            pri_key = Some(PKey::from_ec_key(pri_key_raw)?);
        }

        Ok(JwtEcKeyInner {
            kid: public_jwk.thumbprint_sha256(),
            public: PKey::from_ec_key(pub_key)?,
            public_pem,
            public_jwk,
            private: pri_key,
            private_jwk,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_param() {
        let key = JwtEcKeyInner::from_params(ECurve::ECurveP256, "97137674545862414485372913106719518332096267315416902201730528904183731761874", "87426651606086701008629633665535398077092750158068373881855238506228433010151", Some("90625108496000347380258537061455589299070717029212144039681325579070783497755"), true);
        assert!(key.is_ok());
        let key = key.unwrap();
        assert_eq!(Some(JwkRepr { crv: Some(ECurve::ECurveP256), d: None, dp: None, dq: None, e: None, k: None, kty: Some(JKeyType::JTypeEc), n: None, oth: None, p: Some("90625108496000347380258537061455589299070717029212144039681325579070783497755".to_string()), q: None, qi: None, r: None, t: None, x: Some("97137674545862414485372913106719518332096267315416902201730528904183731761874".to_string()), y: Some("87426651606086701008629633665535398077092750158068373881855238506228433010151".to_string()) }).as_ref(), key.private_key_jwk());

        let key = JwtEcKeyInner::from_params(ECurve::ECurveP256, "97137674545862414485372913106719518332096267315416902201730528904183731761874", "87426651606086701008629633665535398077092750158068373881855238506228433010151", Some("90625108496000347380258537061455589299070717029212144039681325579070783497755"), false);
        assert!(key.is_ok());
        let key = key.unwrap();
        assert_eq!(None, key.private_key_jwk());
    }

    #[test]
    fn test_basics_full() {
        let key = JwtEcKeyInner::generate(ECurve::ECurveP256, false).unwrap();
        assert_eq!(true, key.has_private());
        assert_eq!(false, key.is_exportable());
        assert_eq!(None, key.private_key_jwk());
        let msg = "hello!".as_bytes();
        let sig = key.sign_data(msg).unwrap();
        let ans = key.verify_data(msg, &sig);
        println!("{:?}", key.private_key_jwk());
        assert!(ans.is_ok());
    }
}