use crate::prelude::*;
use jsonwebtoken::encode as jwt_encode;
use jsonwebtoken::Header as JwtHeader;
use jsonwebtoken::{DecodingKey, EncodingKey};
use openssl::bn::{BigNum, BigNumContext};
use openssl::ec::{EcGroup, EcKey};
use openssl::nid::Nid;
use openssl::pkey::PKey;
use ring::digest::{digest, SHA256};

const DEFAULT_JWT_ALG: JwtAlgorithm = JwtAlgorithm::ES256;

#[post("/keys")]
async fn keys_endpoint(data: web::Data<AppState<'_>>, _req: HttpRequest) -> FResult<String> {
    let ans = format!("[{}]", data.jwt.public_key_jwk());
    Ok(ans)
}

// TOOD: (long term) Automatic key rotation

#[derive(Debug, Clone)]
pub struct JwtMaker<'a> {
    alg: JwtAlgorithm,
    kid: Option<String>,
    jku: Option<String>,
    enc_key: EncodingKey,
    dec_key: DecodingKey<'a>,
    dec_jwk: String,
    dec_pem: String,
}

fn alg2nid(alg: JwtAlgorithm) -> FResult<Nid> {
    match alg {
        JwtAlgorithm::ES256 => Ok(Nid::X9_62_PRIME256V1),
        JwtAlgorithm::ES384 => Ok(Nid::SECP384R1),
        // JwtAlgorithm::ES521 => Ok(Nid::SECP521R1),
        _ => Err(FError::FauxPanic(
            "umimplemented JWT Algorithm",
            Some(format!("{:?}", alg)),
        )),
    }
}

fn nid2crv(nid: Nid) -> FResult<&'static str> {
    match nid {
        Nid::X9_62_PRIME256V1 => Ok("P-256"),
        Nid::SECP256K1 => Ok("P-256"),
        Nid::SECP384R1 => Ok("P-384"),
        Nid::SECP521R1 => Ok("P-512"), // not tested
        _ => Err(FError::FauxPanic(
            "umimplemented NID",
            Some(format!("{:?}", nid)),
        )),
    }
}

fn ec_key2jwk<T: openssl::pkey::HasPublic>(ec_key: &EcKey<T>) -> FResult<String> {
    // Get key parameters
    let ec_group = ec_key.group();
    let crv = match ec_group.curve_name() {
        Some(v) => v,
        None => {
            return Err(FError::FauxPanic(
                "failed to compute key thumbprint: EcGroup.curve_name() returned None",
                None,
            ))
        }
    };
    let crv = nid2crv(crv)?;
    let point = ec_key.public_key();
    let mut bigctx = BigNumContext::new()?;
    let mut x = BigNum::new()?;
    let mut y = BigNum::new()?;
    point.affine_coordinates_gfp(ec_group, &mut x, &mut y, &mut bigctx)?;

    // Make the "canonical" JSON representation of the key
    let jk = format!(
        "{{\"crv\":\"{}\",\"kty\":\"EC\",\"x\":\"{}\",\"y\":\"{}\"}}",
        crv, x, y
    );
    debug!("{}", jk);
    Ok(jk)
}

fn compute_sha256_base64(val: &str) -> String {
    // And use the hash of said representation as the kid
    let hash = digest(&SHA256, val.as_bytes());
    base64::encode(hash)
}

impl JwtMaker<'_> {
    pub fn new<'a>() -> FResult<JwtMaker<'a>> {
        JwtMaker::new_alg(DEFAULT_JWT_ALG)
    }

    pub fn new_alg<'a>(alg: JwtAlgorithm) -> FResult<JwtMaker<'a>> {
        JwtMaker::new_ec(alg)
    }

    fn new_ec<'a>(alg: JwtAlgorithm) -> FResult<JwtMaker<'a>> {
        let alg_nid = alg2nid(alg)?;
        let ecg = EcGroup::from_curve_name(alg_nid)?;
        let ec_key = EcKey::generate(&ecg)?;
        let key = PKey::from_ec_key(ec_key.clone())?;
        let pem_private = key.private_key_to_pem_pkcs8()?;
        let pem_public = key.public_key_to_pem()?;
        let enc_key = EncodingKey::from_ec_pem(&pem_private)?;
        let dec_key = DecodingKey::from_ec_pem(&pem_public)?;
        let dec_key = dec_key.into_static();

        let dec_jwk = ec_key2jwk(&ec_key)?;
        let dec_pem = match std::str::from_utf8(&pem_public) {
            Ok(v) => v,
            Err(err) => {
                return Err(FError::FauxPanic(
                    "failed to load PEM string",
                    Some(format!("{:?}", err)),
                ))
            }
        };

        let kid = compute_sha256_base64(&dec_jwk);

        let ans = JwtMaker {
            alg: alg,
            kid: Some(kid),
            jku: None,
            enc_key: enc_key,
            dec_key: dec_key,
            dec_jwk: dec_jwk,
            dec_pem: dec_pem.to_string(),
        };
        if let Err(err) = ans.issue("hi") {
            let msg = format!("{:?}", err);
            return Err(FError::FauxPanic("failed to issue test token", Some(msg)));
        }
        Ok(ans)
    }

    pub fn public_key_jwk(&self) -> &str {
        &self.dec_jwk
    }

    pub fn public_key_pem(&self) -> &str {
        &self.dec_pem
    }

    pub fn issue(&self, claims: impl Serialize) -> FResult<String> {
        let mut header = JwtHeader::new(self.alg);
        header.kid = self.kid.clone();
        header.jku = self.jku.clone();
        Ok(jwt_encode(&header, &claims, &self.enc_key)?)
    }
}
