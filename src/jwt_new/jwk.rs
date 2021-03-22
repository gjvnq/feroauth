use crate::jwt_new::prelude::*;
use ring::digest::{digest, SHA256};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(into = "&str", try_from = "&str")]
pub enum JwkUse {
    Sig,
    Enc,
}

impl JwkUse {
    fn to_str(&self) -> &'static str {
        match self {
            JwkUse::Sig => "sig",
            JwkUse::Enc => "enc",
        }
    }
}

impl std::convert::From<JwkUse> for &str {
    fn from(val: JwkUse) -> Self {
        val.to_str()
    }
}

impl std::convert::TryFrom<&str> for JwkUse {
    type Error = JwtError;

    fn try_from(val: &str) -> Result<Self, JwtError> {
        match val {
            "sig" => Ok(JwkUse::Sig),
            "enc" => Ok(JwkUse::Enc),
            _ => Err(JwtError::new(JwtErrorInner::UnknownKeyUse(val.to_string()))),
        }
    }
}

impl std::fmt::Display for JwkUse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(into = "&str", try_from = "&str")]
pub enum JKeyType {
    JTypeEc,
    JTypeRsa,
    JTypeOct,
}

use JKeyType::{JTypeEc, JTypeOct, JTypeRsa};

impl JKeyType {
    fn to_str(&self) -> &'static str {
        match self {
            JTypeEc => "EC",
            JTypeRsa => "RSA",
            JTypeOct => "oct",
        }
    }
}

impl std::convert::From<JKeyType> for &str {
    fn from(val: JKeyType) -> Self {
        val.to_str()
    }
}

impl std::convert::TryFrom<&str> for JKeyType {
    type Error = JwtError;

    fn try_from(val: &str) -> Result<Self, JwtError> {
        match val {
            "EC" => Ok(JTypeEc),
            "RSA" => Ok(JTypeRsa),
            "oct" => Ok(JTypeOct),
            _ => Err(JwtError::new(JwtErrorInner::UnknownKeyType(
                val.to_string(),
            ))),
        }
    }
}

impl std::fmt::Display for JKeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(into = "&str", try_from = "&str")]
pub enum ECurve {
    ECurveP256,
    ECurveP384,
    ECurveP521,
}

use {ECurve::ECurveP256, ECurve::ECurveP384, ECurve::ECurveP521};

impl ECurve {
    pub fn to_str(&self) -> &'static str {
        match self {
            ECurveP256 => "P-256",
            ECurveP384 => "P-384",
            ECurveP521 => "P-521",
        }
    }

    pub fn to_nid(&self) -> Nid {
        match self {
            ECurveP256 => Nid::X9_62_PRIME256V1,
            ECurveP384 => Nid::SECP384R1,
            ECurveP521 => Nid::SECP521R1,
        }
    }

    pub fn to_alg(&self) -> JwtAlgorithm {
        match self {
            ECurveP256 => JwtAlgorithm::ES256,
            ECurveP384 => JwtAlgorithm::ES384,
            ECurveP521 => JwtAlgorithm::ES512,
        }
    }
}

impl std::fmt::Display for ECurve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl std::convert::From<ECurve> for &str {
    fn from(val: ECurve) -> Self {
        val.to_str()
    }
}

impl std::convert::TryFrom<&str> for ECurve {
    type Error = JwtError;

    fn try_from(val: &str) -> Result<Self, JwtError> {
        match val {
            "P-256" => Ok(ECurveP256),
            "P-384" => Ok(ECurveP384),
            "P-521" => Ok(ECurveP521),
            _ => Err(JwtError::new(JwtErrorInner::UnknownCurve(val.to_string()))),
        }
    }
}

impl std::convert::TryFrom<Nid> for ECurve {
    type Error = JwtError;

    fn try_from(val: Nid) -> Result<Self, JwtError> {
        match val {
            Nid::X9_62_PRIME256V1 => Ok(ECurveP256),
            Nid::SECP256K1 => Ok(ECurveP256),
            Nid::SECP384R1 => Ok(ECurveP384),
            Nid::SECP521R1 => Ok(ECurveP521),
            _ => Err(JwtError::new(JwtErrorInner::UnknownCurve(format!(
                "{:?}",
                val
            )))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(into = "&str", try_from = "&str")]
pub enum JwtAlgorithm {
    /// MAC using SHA-256
    HS256,
    /// HMAC using SHA-384
    HS384,
    /// HMAC using SHA-512
    HS512,
    /// ECDSA using P-256 and SHA-256
    ES256,
    /// ECDSA using P-384 and SHA-384
    ES384,
    /// ECDSA using P-521 and SHA-512
    ES512,
    /// RSASSA-PKCS1-v1_5 using SHA-256
    RS256,
    /// RSASSA-PKCS1-v1_5 using SHA-384
    RS384,
    /// RSASSA-PKCS1-v1_5 using SHA-512
    RS512,
    /// RSASSA-PSS using SHA-256 and MGF1 with SHA-256
    PS256,
    /// RSASSA-PSS using SHA-384 and MGF1 with SHA-384
    PS384,
    /// RSASSA-PSS using SHA-512 and MGF1 with SHA-512
    PS512,
}

impl std::fmt::Display for JwtAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl std::convert::From<JwtAlgorithm> for &str {
    fn from(val: JwtAlgorithm) -> Self {
        val.to_str()
    }
}

impl JwtAlgorithm {
    fn to_str(&self) -> &'static str {
        match self {
            JwtAlgorithm::HS256 => "HS256",
            JwtAlgorithm::HS384 => "HS384",
            JwtAlgorithm::HS512 => "HS512",
            JwtAlgorithm::ES256 => "ES256",
            JwtAlgorithm::ES384 => "ES384",
            JwtAlgorithm::ES512 => "ES512",
            JwtAlgorithm::RS256 => "RS256",
            JwtAlgorithm::RS384 => "RS384",
            JwtAlgorithm::RS512 => "RS512",
            JwtAlgorithm::PS256 => "PS256",
            JwtAlgorithm::PS384 => "PS384",
            JwtAlgorithm::PS512 => "PS512",
        }
    }

    pub(crate) fn to_hasher(&self) -> JwtResult<SslHasher> {
        Ok(SslHasher::new(self.to_md())?)
    }

    pub(crate) fn to_md(&self) -> MessageDigest {
        match self {
            JwtAlgorithm::HS256 => MessageDigest::sha256(),
            JwtAlgorithm::HS384 => MessageDigest::sha384(),
            JwtAlgorithm::HS512 => MessageDigest::sha512(),
            JwtAlgorithm::ES256 => MessageDigest::sha256(),
            JwtAlgorithm::ES384 => MessageDigest::sha384(),
            JwtAlgorithm::ES512 => MessageDigest::sha512(),
            JwtAlgorithm::RS256 => MessageDigest::sha256(),
            JwtAlgorithm::RS384 => MessageDigest::sha384(),
            JwtAlgorithm::RS512 => MessageDigest::sha512(),
            JwtAlgorithm::PS256 => MessageDigest::sha256(),
            JwtAlgorithm::PS384 => MessageDigest::sha384(),
            JwtAlgorithm::PS512 => MessageDigest::sha512(),
        }
    }
}

impl std::convert::TryFrom<&str> for JwtAlgorithm {
    type Error = JwtError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            "HS256" => Ok(JwtAlgorithm::HS256),
            "HS384" => Ok(JwtAlgorithm::HS384),
            "HS512" => Ok(JwtAlgorithm::HS512),
            "ES256" => Ok(JwtAlgorithm::ES256),
            "ES384" => Ok(JwtAlgorithm::ES384),
            "ES512" => Ok(JwtAlgorithm::ES512),
            "RS256" => Ok(JwtAlgorithm::RS256),
            "RS384" => Ok(JwtAlgorithm::RS384),
            "RS512" => Ok(JwtAlgorithm::RS512),
            "PS256" => Ok(JwtAlgorithm::PS256),
            "PS384" => Ok(JwtAlgorithm::PS384),
            "PS512" => Ok(JwtAlgorithm::PS512),
            _ => Err(JwtError::new(JwtErrorInner::UnknownAlg(val.to_string()))),
        }
    }
}

impl std::convert::From<JwtAlgorithm> for JKeyType {
    fn from(val: JwtAlgorithm) -> Self {
        match val {
            JwtAlgorithm::HS256 => JKeyType::JTypeOct,
            JwtAlgorithm::HS384 => JKeyType::JTypeOct,
            JwtAlgorithm::HS512 => JKeyType::JTypeOct,
            JwtAlgorithm::ES256 => JKeyType::JTypeEc,
            JwtAlgorithm::ES384 => JKeyType::JTypeEc,
            JwtAlgorithm::ES512 => JKeyType::JTypeEc,
            JwtAlgorithm::RS256 => JKeyType::JTypeRsa,
            JwtAlgorithm::RS384 => JKeyType::JTypeRsa,
            JwtAlgorithm::RS512 => JKeyType::JTypeRsa,
            JwtAlgorithm::PS256 => JKeyType::JTypeRsa,
            JwtAlgorithm::PS384 => JKeyType::JTypeRsa,
            JwtAlgorithm::PS512 => JKeyType::JTypeRsa,
        }
    }
}

impl std::convert::TryFrom<JwtAlgorithm> for ECurve {
    type Error = JwtError;
    #[track_caller]
    fn try_from(val: JwtAlgorithm) -> Result<Self, Self::Error> {
        match val {
            JwtAlgorithm::ES256 => Ok(ECurve::ECurveP256),
            JwtAlgorithm::ES384 => Ok(ECurve::ECurveP384),
            JwtAlgorithm::ES512 => Ok(ECurve::ECurveP521),
            _ => Err(JwtError::new(JwtErrorInner::AlgHasNoCurveType(val))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct JwkRepr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crv: Option<ECurve>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dq: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub k: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kty: Option<JKeyType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
}

impl JwkRepr {
    // cannnical json (note this will produce garbage if the key is "misconfigured")
    pub fn rfc_7638(&self) -> String {
        let mut ans = String::from("{");

        fn add_kv(ans: &mut String, k: &str, v: &str) {
            if ans.len() > 1 {
                ans.push(',');
            }
            ans.push('"');
            ans.push_str(k);
            ans.push_str("\":\"");
            ans.push_str(v);
            ans.push('"');
        }

        if let Some(crv) = &self.crv {
            add_kv(&mut ans, "crv", &crv.to_str());
        }
        if let Some(e) = &self.e {
            add_kv(&mut ans, "e", &e);
        }
        if let Some(k) = &self.k {
            add_kv(&mut ans, "k", &k);
        }
        if let Some(kty) = &self.kty {
            add_kv(&mut ans, "kty", &kty.to_str());
        }
        if let Some(n) = &self.n {
            add_kv(&mut ans, "n", &n);
        }
        if let Some(x) = &self.x {
            add_kv(&mut ans, "x", &x);
        }
        if let Some(y) = &self.y {
            add_kv(&mut ans, "y", &y);
        }
        ans.push('}');
        ans
    }

    pub fn thumbprint_sha256(&self) -> String {
        let cannonical = self.rfc_7638();
        let hash = digest(&SHA256, cannonical.as_bytes());
        base64::encode(hash)
    }

    // Checks if there is missing information. Returns false if something is missing
    pub fn has_necessary_fields(&self) -> bool {
        match self.kty {
            Some(JTypeEc) => self.crv.is_some() && self.x.is_some() && self.y.is_some(),
            Some(JTypeRsa) => self.e.is_some() && self.n.is_some(),
            Some(JTypeOct) => self.k.is_some(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jwk_json_encode_1() {
        let key = JwkRepr {
            kty: Some(JTypeRsa),
            ..Default::default()
        };
        let j = serde_json::to_string(&key).unwrap();
        println!("{:?}", j);
        assert_eq!(r#"{"kty":"RSA"}"#, j);
        assert_eq!(r#"{"kty":"RSA"}"#, key.rfc_7638());
        assert_eq!(false, key.has_necessary_fields());
    }

    #[test]
    fn jwk_json_encode_2() {
        let key = JwkRepr {
            kty: Some(JTypeEc),
            crv: Some(ECurveP256),
            ..Default::default()
        };
        let j = serde_json::to_string(&key).unwrap();
        println!("{:?}", j);
        assert_eq!(r#"{"crv":"P-256","kty":"EC"}"#, j);
        assert_eq!(false, key.has_necessary_fields());
    }

    #[test]
    fn jwk_json_decode_1() {
        let raw = r#"{"crv":"P-256","kty":"EC","x":"7499149945062600705050524764908008277809478315939756347263185989105148022349","y":"69170000752570775739841997346942906979532601058330588010566802477329443185253"}"#;
        let key = serde_json::from_str::<JwkRepr>(raw).unwrap();
        println!("{:?}", key);
        assert_eq!(
            key,
            JwkRepr {
                kty: Some(JTypeEc),
                crv: Some(ECurveP256),
                x: Some(
                    "7499149945062600705050524764908008277809478315939756347263185989105148022349"
                        .to_string()
                ),
                y: Some(
                    "69170000752570775739841997346942906979532601058330588010566802477329443185253"
                        .to_string()
                ),
                ..Default::default()
            }
        );
        assert_eq!(raw, key.rfc_7638());
        assert_eq!(true, key.has_necessary_fields());
    }
}
