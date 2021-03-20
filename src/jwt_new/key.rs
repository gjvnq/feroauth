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
pub struct JwtEcKeyInner {
    public: SslEcKey<Public>,
    public_pem: String,
    public_jwk: JwkRepr,
    private: Option<SslEcKey<Private>>,
    private_jwk: Option<JwkRepr>,
}
#[derive(Debug, Clone)]
pub struct JwtRsaKeyInner {
    public: SslRsaKey<Public>,
    private: Option<SslRsaKey<Private>>,
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

    /* HMACSHA256(  base64UrlEncode(header) + "." +  base64UrlEncode(payload),  secret) */
    fn sign_data(&self, data: &str) -> JwtResult<String>;
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
    fn sign_data(&self, data: &str) -> JwtResult<String> {
        todo!()
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

    fn get_crv_x_y(key: &SslEcKey<Public>) -> JwtResult<(JCurveType, String, String)> {
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

        let crv = JCurveType::try_from(crv)?;
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

    #[allow(unused)]
    pub(crate) fn generate(crv: &JCurveType, exportable: bool) -> JwtResult<Self> {
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
            public: pub_key,
            public_jwk,
            public_pem,
            private: Some(pri_key),
            private_jwk,
        })
    }

    #[allow(unused)]
    pub(crate) fn from_params(
        crv: &JCurveType,
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
            pri_key = Some(pri_key_raw);
        }

        Ok(JwtEcKeyInner {
            public: pub_key,
            public_pem,
            public_jwk,
            private: pri_key,
            private_jwk,
        })
    }
}
