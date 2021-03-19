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
async fn keys_endpoint(_data: web::Data<AppState<'_>>, _req: HttpRequest) -> FResult<HttpResponse> {
    unimplemented!()
}

// TOOD: Automatic key rotation

#[derive(Debug, Clone)]
pub struct JwtMaker<'a> {
    alg: JwtAlgorithm,
    kid: Option<String>,
    jku: Option<String>,
    enc_key: EncodingKey,
    dec_key: DecodingKey<'a>,
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

fn compute_kid_ec<T: openssl::pkey::HasPublic>(ec_key: &EcKey<T>) -> FResult<String> {
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
    let mut bigctx = match BigNumContext::new() {
        Ok(v) => v,
        Err(err) => {
            let msg = format!("error_stack={:?}", err);
            return Err(FError::FauxPanic(
                "failed to compute key thumbprint: BigNumContext::new",
                Some(msg),
            ));
        }
    };
    let mut x = match BigNum::new() {
        Ok(v) => v,
        Err(err) => {
            let msg = format!("error_stack={:?}", err);
            return Err(FError::FauxPanic(
                "failed to compute key thumbprint: BigNum::new",
                Some(msg),
            ));
        }
    };
    let mut y = match BigNum::new() {
        Ok(v) => v,
        Err(err) => {
            let msg = format!("error_stack={:?}", err);
            return Err(FError::FauxPanic(
                "failed to compute key thumbprint: BigNum::new",
                Some(msg),
            ));
        }
    };
    if let Err(err) = point.affine_coordinates_gfp(ec_group, &mut x, &mut y, &mut bigctx) {
        let msg = format!("error_stack={:?}", err);
        return Err(FError::FauxPanic(
            "failed to compute key thumbprint: EcPointRef.affine_coordinates_gfp",
            Some(msg),
        ));
    }

    // Make the "canonical" JSON representation of the key
    let jk = format!(
        "{{\"crv\":\"{}\",\"kty\":\"EC\",\"x\":\"{}\",\"y\":\"{}\"}}",
        crv, x, y
    );
    debug!("{}", jk);

    // And use the hash of said representation as the kid
    let hash = digest(&SHA256, jk.as_bytes());
    Ok(base64::encode(hash))
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
        let ecg = match EcGroup::from_curve_name(alg_nid) {
            Ok(v) => v,
            Err(err) => {
                let msg = format!("alg={:?} error_stack={:?}", alg, err);
                return Err(FError::FauxPanic(
                    "failed to generate JWT key pair: EcGroup::from_curve_name",
                    Some(msg),
                ));
            }
        };
        let ec_key = match EcKey::generate(&ecg) {
            Ok(v) => v,
            Err(err) => {
                let msg = format!("alg={:?} error_stack={:?}", alg, err);
                return Err(FError::FauxPanic(
                    "failed to generate JWT key pair: EcKey::generate",
                    Some(msg),
                ));
            }
        };
        let key = match PKey::from_ec_key(ec_key.clone()) {
            Ok(v) => v,
            Err(err) => {
                let msg = format!("alg={:?} error_stack={:?}", alg, err);
                return Err(FError::FauxPanic(
                    "failed to generate JWT key pair: PKey::from_ec_key",
                    Some(msg),
                ));
            }
        };
        let pem_private = match key.private_key_to_pem_pkcs8() {
            Ok(v) => v,
            Err(err) => {
                let msg = format!("alg={:?} error_stack={:?}", alg, err);
                return Err(FError::FauxPanic(
                    "failed to generate JWT key pair: key.private_key_to_pem_pkcs8",
                    Some(msg),
                ));
            }
        };
        let pem_public = match key.public_key_to_pem() {
            Ok(v) => v,
            Err(err) => {
                let msg = format!("alg={:?} error_stack={:?}", alg, err);
                return Err(FError::FauxPanic(
                    "failed to generate JWT key pair: key.public_key_to_pem",
                    Some(msg),
                ));
            }
        };
        let enc_key = match EncodingKey::from_ec_pem(&pem_private) {
            Ok(v) => v,
            Err(err) => {
                let msg = format!("alg={:?} error_stack={:?}", alg, err);
                return Err(FError::FauxPanic(
                    "failed to generate JWT key pair: EncodingKey::from_ec_pem",
                    Some(msg),
                ));
            }
        };
        let dec_key = match DecodingKey::from_ec_pem(&pem_public) {
            Ok(v) => v,
            Err(err) => {
                let msg = format!("alg={:?} error_stack={:?}", alg, err);
                return Err(FError::FauxPanic(
                    "failed to generate JWT key pair: DecodingKey::from_ec_pem",
                    Some(msg),
                ));
            }
        };
        let dec_key = dec_key.into_static();

        let kid = compute_kid_ec(&ec_key)?;
        debug!(
            "PEM pub key for {}: {}",
            kid,
            std::str::from_utf8(&pem_public)
                .unwrap_or("")
                .replace("-----\n", "----- ")
                .replace("\n-----", " -----")
                .replace("\n", "")
        );

        let ans = JwtMaker {
            alg: alg,
            kid: Some(kid),
            jku: None,
            enc_key: enc_key,
            dec_key: dec_key,
        };
        if let Err(err) = ans.issue("hi") {
            let msg = format!("{:?}", err);
            return Err(FError::FauxPanic("failed to issue test token", Some(msg)));
        }
        Ok(ans)
    }

    pub fn issue(&self, claims: impl Serialize) -> FResult<String> {
        let mut header = JwtHeader::new(self.alg);
        header.kid = self.kid.clone();
        header.jku = self.jku.clone();
        Ok(jwt_encode(&header, &claims, &self.enc_key)?)
    }
}
