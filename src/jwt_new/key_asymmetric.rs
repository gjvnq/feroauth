use crate::jwt_new::prelude::*;
use crate::jwt_new::*;
use openssl::rsa::RsaPrivateKeyBuilder;

const DEFAULT_RSA_KEY_SIZE: u32 = 2048;

#[derive(Debug, Clone)]
pub struct JwtAsymmetricKeyInner {
    alg: JwtAlgorithm,
    kind: JwkUse,
    thumbprint_sha256: String,
    key_pk: PKey<Public>,
    pem_pk: String,
    jwk_pem: JwkRepr,
    key_sk: Option<PKey<Private>>,
    jwk_sk: Option<JwkRepr>,
}

impl JwKeyTraitLowLevel for JwtAsymmetricKeyInner {
    fn algorithm(&self) -> JwtAlgorithm {
        self.algorithm()
    }
    fn kind(&self) -> JwkUse {
        self.kind()
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
        self.thumbprint_sha256()
    }
    fn key_type(&self) -> JKeyType {
        self.key_type()
    }
    fn private_key_jwk(&self) -> Option<&JwkRepr> {
        self.jwk_sk.as_ref()
    }
}

impl JwtAsymmetricKeyInner {
    pub fn algorithm(&self) -> JwtAlgorithm {
        self.alg
    }

    pub fn kind(&self) -> JwkUse {
        self.kind
    }

    pub fn key_type(&self) -> JKeyType {
        self.jwk_pem.kty.unwrap()
    }

    pub fn thumbprint_sha256(&self) -> &str {
        &self.thumbprint_sha256
    }

    pub fn public_key_jwk(&self) -> &JwkRepr {
        &self.jwk_pem
    }

    pub fn public_key_pem(&self) -> &str {
        &self.pem_pk
    }

    pub fn private_key_jwk(&self) -> Option<&JwkRepr> {
        // if the key is non exportable, this field would be blank
        self.jwk_sk.as_ref()
    }

    pub fn has_private(&self) -> bool {
        self.key_sk.is_some()
    }

    pub fn is_exportable(&self) -> bool {
        self.jwk_sk.is_some()
    }

    fn hash(&self, data: &[u8]) -> JwtResult<Vec<u8>> {
        let mut hasher = self.algorithm().to_hasher()?;
        hasher.update(data)?;
        Ok(hasher.finish()?.to_vec())
    }

    fn sign_data(&self, data: &[u8]) -> JwtResult<Vec<u8>> {
        let sk = match &self.key_sk {
            Some(key) => key,
            None => {
                return Err(JwtError::new(JwtErrorInner::NoPrivateKeyForPubKey(
                    self.thumbprint_sha256.clone(),
                )))
            }
        };
        let mut signer = SslSigner::new(self.algorithm().to_md(), sk)?;
        let ans = signer.sign_oneshot_to_vec(data)?;

        if self.key_type() == JKeyType::JTypeEc {
            Ok(ecdsa_der2plain(&ans)?)
        } else {
            Ok(ans)
        }
    }

    fn verify_data(&self, data: &[u8], sig: &[u8]) -> JwtResult<()> {
        let sig = match self.key_type() {
            JKeyType::JTypeEc => ecdsa_plain2der(sig)?,
            _ => sig.to_vec(),
        };

        let pk = &self.key_pk;
        let mut verifier = SslVerifier::new(self.algorithm().to_md(), pk)?;
        match verifier.verify_oneshot(&sig, data)? {
            true => Ok(()),
            false => {
                // pretty print stuff and return the error
                let hash = self
                    .hash(data)
                    .map(|v| hex::encode(v))
                    .unwrap_or(base64::encode(data));
                let sig = base64::encode(sig);
                Err(JwtError::new(JwtErrorInner::InvalidSignature {
                    kid: self.thumbprint_sha256.clone(),
                    data: hash,
                    sig: sig,
                }))
            }
        }
    }

    fn ec_get_crv_x_y(key: &SslEcKey<Public>) -> JwtResult<(ECurve, String, String)> {
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
        Ok((crv, bn_to_b64(&x), bn_to_b64(&y)))
    }

    fn ec_make_jwk_public(key: &SslEcKey<Public>) -> JwtResult<JwkRepr> {
        let (crv, x, y) = JwtAsymmetricKeyInner::ec_get_crv_x_y(key)?;
        Ok(JwkRepr {
            kty: Some(JKeyType::JTypeEc),
            crv: Some(crv),
            x: Some(x),
            y: Some(y),
            ..Default::default()
        })
    }

    fn ec_make_jwk_private(ans: &JwkRepr, key: &SslEcKey<Private>) -> JwkRepr {
        let mut ans = ans.clone();
        ans.d = Some(bn_to_b64(key.private_key()));
        ans
    }

    fn rsa_make_jwk_public(key: &SslRsaKey<Public>) -> JwkRepr {
        JwkRepr {
            kty: Some(JKeyType::JTypeRsa),
            e: Some(bn_to_b64(key.e())),
            n: Some(bn_to_b64(key.n())),
            ..Default::default()
        }
    }

    fn rsa_make_jwk_private(key: &SslRsaKey<Private>) -> JwkRepr {
        JwkRepr {
            kty: Some(JKeyType::JTypeRsa),
            n: Some(bn_to_b64(key.n())),
            e: Some(bn_to_b64(key.e())),
            d: Some(bn_to_b64(key.d())),
            p: option_bn_to_b64(key.p()),
            q: option_bn_to_b64(key.q()),
            dp: option_bn_to_b64(key.dmp1()),
            dq: option_bn_to_b64(key.dmq1()),
            qi: option_bn_to_b64(key.iqmp()),
            ..Default::default()
        }
    }

    pub fn generate(alg: JwtAlgorithm, exportable: bool) -> JwtResult<Self> {
        match alg {
            JwtAlgorithm::ES256 => {
                JwtAsymmetricKeyInner::generate_ec(ECurve::try_from(alg).unwrap(), exportable)
            }
            JwtAlgorithm::ES384 => {
                JwtAsymmetricKeyInner::generate_ec(ECurve::try_from(alg).unwrap(), exportable)
            }
            JwtAlgorithm::ES512 => {
                JwtAsymmetricKeyInner::generate_ec(ECurve::try_from(alg).unwrap(), exportable)
            }
            JwtAlgorithm::RS256 => {
                JwtAsymmetricKeyInner::generate_rsa(alg, DEFAULT_RSA_KEY_SIZE, exportable)
            }
            JwtAlgorithm::RS384 => {
                JwtAsymmetricKeyInner::generate_rsa(alg, DEFAULT_RSA_KEY_SIZE, exportable)
            }
            JwtAlgorithm::RS512 => {
                JwtAsymmetricKeyInner::generate_rsa(alg, DEFAULT_RSA_KEY_SIZE, exportable)
            }
            _ => {
                return Err(JwtError::new(JwtErrorInner::NotImplemented(format!(
                    "generate key for alg={}",
                    alg
                ))))
            }
        }
    }

    pub(crate) fn generate_rsa(alg: JwtAlgorithm, bits: u32, exportable: bool) -> JwtResult<Self> {
        let rsa_sk = SslRsaKey::generate(bits)?;
        let jwk_sk = match exportable {
            true => Some(JwtAsymmetricKeyInner::rsa_make_jwk_private(&rsa_sk)),
            false => None,
        };
        let rsa_pk =
            SslRsaKey::from_public_components(rsa_sk.n().to_owned()?, rsa_sk.e().to_owned()?)?;
        let jwk_pk = JwtAsymmetricKeyInner::rsa_make_jwk_public(&rsa_pk);
        let pem_pk = String::from_utf8(rsa_pk.public_key_to_pem()?)?;

        Ok(JwtAsymmetricKeyInner {
            alg: alg,
            kind: JwkUse::Sig,
            thumbprint_sha256: jwk_pk.thumbprint_sha256(),
            key_pk: PKey::from_rsa(rsa_pk)?,
            jwk_pem: jwk_pk,
            pem_pk: pem_pk,
            key_sk: Some(PKey::from_rsa(rsa_sk)?),
            jwk_sk: jwk_sk,
        })
    }

    pub(crate) fn from_params_rsa(
        alg: JwtAlgorithm,
        n: &str,
        e: &str,
        d: Option<&str>,
        p: Option<&str>,
        q: Option<&str>,
        dp: Option<&str>,
        dq: Option<&str>,
        qi: Option<&str>,
        exportable: bool,
    ) -> JwtResult<Self> {
        let n = b64_to_bn(n)?;
        let e = b64_to_bn(e)?;
        let d = option_b64_to_bn(d)?;
        let p = option_b64_to_bn(p)?;
        let q = option_b64_to_bn(q)?;
        let dp = option_b64_to_bn(dp)?;
        let dq = option_b64_to_bn(dq)?;
        let qi = option_b64_to_bn(qi)?;

        let rsa_pk = SslRsaKey::from_public_components(n.to_owned()?, e.to_owned()?)?;
        let jwk_pk = JwtAsymmetricKeyInner::rsa_make_jwk_public(&rsa_pk);
        let pem_pk = String::from_utf8(rsa_pk.public_key_to_pem()?)?;

        let (key_sk, jwk_sk) = match d {
            Some(d) => {
                let mut builder = RsaPrivateKeyBuilder::new(n.to_owned()?, e.to_owned()?, d)?;
                if p.is_some() && q.is_some() {
                    let (p, q) = (p.unwrap(), q.unwrap());

                    builder = builder.set_factors(p, q)?;
                    if dp.is_some() && dq.is_some() && qi.is_some() {
                        let (dp, dq, qi) = (dp.unwrap(), dq.unwrap(), qi.unwrap());

                        builder = builder.set_crt_params(dp, dq, qi)?;
                    }
                }
                let rsa_sk = builder.build();
                let okk_sk = rsa_sk.check_key();
                if let Err(err) = okk_sk {
                    return Err(JwtError::new(JwtErrorInner::InvalidKey(format!(
                        "RSA n={} e={} err={:?}",
                        n, e, err
                    ))));
                }
                if okk_sk.unwrap() == false {
                    return Err(JwtError::new(JwtErrorInner::InvalidKey(format!(
                        "RSA n={} e={}",
                        n, e
                    ))));
                }
                let jwk_sk = match exportable {
                    true => Some(JwtAsymmetricKeyInner::rsa_make_jwk_private(&rsa_sk)),
                    false => None,
                };
                let key_sk = PKey::from_rsa(rsa_sk)?;

                (Some(key_sk), jwk_sk)
            }
            None => ((None, None)),
        };

        Ok(JwtAsymmetricKeyInner {
            alg: alg,
            kind: JwkUse::Sig,
            thumbprint_sha256: jwk_pk.thumbprint_sha256(),
            key_pk: PKey::from_rsa(rsa_pk)?,
            jwk_pem: jwk_pk,
            pem_pk: pem_pk,
            key_sk: key_sk,
            jwk_sk: jwk_sk,
        })
    }

    pub(crate) fn generate_ec(crv: ECurve, exportable: bool) -> JwtResult<Self> {
        let ecg_nid = crv.to_nid();
        let ecg = EcGroup::from_curve_name(ecg_nid)?;
        let ecc_sk = SslEcKey::generate(&ecg)?;
        let ecc_pk = SslEcKey::from_public_key(&ecg, &ecc_sk.public_key())?;

        if let Err(err) = ecc_pk.check_key() {
            return Err(JwtError::new(JwtErrorInner::InvalidKey(format!(
                "crv={} err={:?}",
                crv, err
            ))));
        }
        if let Err(err) = ecc_sk.check_key() {
            return Err(JwtError::new(JwtErrorInner::InvalidKey(format!(
                "crv={} err={:?}",
                crv, err
            ))));
        }

        let jwk_pk = JwtAsymmetricKeyInner::ec_make_jwk_public(&ecc_pk)?;
        let jwk_sk = match exportable {
            true => Some(JwtAsymmetricKeyInner::ec_make_jwk_private(&jwk_pk, &ecc_sk)),
            false => None,
        };

        let pem_pk = String::from_utf8(ecc_pk.public_key_to_pem()?)?;

        Ok(JwtAsymmetricKeyInner {
            alg: crv.to_alg(),
            kind: JwkUse::Sig,
            thumbprint_sha256: jwk_pk.thumbprint_sha256(),
            key_pk: PKey::from_ec_key(ecc_pk)?,
            jwk_pem: jwk_pk,
            pem_pk: pem_pk,
            key_sk: Some(PKey::from_ec_key(ecc_sk)?),
            jwk_sk: jwk_sk,
        })
    }

    #[allow(unused)]
    pub(crate) fn from_params_ec(
        crv: ECurve,
        x: &str,
        y: &str,
        pn: Option<&str>,
        exportable: bool,
    ) -> JwtResult<Self> {
        let x = b64_to_bn(x)?;
        let y = b64_to_bn(y)?;

        let ecg_nid = crv.to_nid();
        let ecg = EcGroup::from_curve_name(ecg_nid)?;
        let ecc_key = SslEcKey::from_public_key_affine_coordinates(&ecg, &x, &y)?;

        if let Err(err) = ecc_key.check_key() {
            return Err(JwtError::new(JwtErrorInner::InvalidKey(format!(
                "crv={} x={} y={} err={:?}",
                crv, x, y, err
            ))));
        }

        let jwk_pem = JwtAsymmetricKeyInner::ec_make_jwk_public(&ecc_key)?;
        let pem_pk = String::from_utf8(ecc_key.public_key_to_pem()?)?;

        let mut key_sk = None;
        let mut jwk_sk = None;
        if let Some(pn) = pn {
            // Add private key info if provided by user
            let pn = b64_to_bn(pn)?;
            let point = ecc_key.public_key();
            let ecc_sk = SslEcKey::from_private_components(&ecg, &pn, &point)?;
            if let Err(err) = ecc_sk.check_key() {
                return Err(JwtError::new(JwtErrorInner::InvalidKey(format!(
                    "crv={} x={} y={} p=Some(...) err={:?}",
                    crv, x, y, err
                ))));
            }
            if exportable {
                jwk_sk = Some(JwtAsymmetricKeyInner::ec_make_jwk_private(
                    &jwk_pem, &ecc_sk,
                ));
            }
            key_sk = Some(PKey::from_ec_key(ecc_sk)?);
        }

        Ok(JwtAsymmetricKeyInner {
            alg: crv.to_alg(),
            kind: JwkUse::Sig,
            thumbprint_sha256: jwk_pem.thumbprint_sha256(),
            key_pk: PKey::from_ec_key(ecc_key)?,
            pem_pk,
            jwk_pem,
            key_sk: key_sk,
            jwk_sk,
        })
    }
}

fn ecdsa_der2plain(sig_der: &[u8]) -> JwtResult<Vec<u8>> {
    println!("{:?}", sig_der);
    println!("{:?}", base64::encode(sig_der));

    let sig_obj = openssl::ecdsa::EcdsaSig::from_der(sig_der)?;

    let r_vec = sig_obj.r().to_vec();
    let s_vec = sig_obj.s().to_vec();

    let mut r_final = vec![0; 32];
    let mut s_final = vec![0; 32];

    let mut i = r_vec.len() - 1;
    let mut j = r_final.len() - 1;
    loop {
        r_final[j] = r_vec[i];
        if i == 0 || j == 0 {
            break;
        }
        i -= 1;
        j -= 1;
    }
    let mut i = s_vec.len() - 1;
    let mut j = s_final.len() - 1;
    loop {
        s_final[j] = s_vec[i];
        if i == 0 || j == 0 {
            break;
        }
        i -= 1;
        j -= 1;
    }

    let plain = [r_final, s_final].concat();

    Ok(plain)
}

fn ecdsa_plain2der(sig_plain: &[u8]) -> JwtResult<Vec<u8>> {
    if sig_plain.len() != 64 {
        return Err(JwtError::new_panic_2(
            "wrong length for EcDSA signature",
            Some(format!("wanted {} got {}", 64, sig_plain.len())),
        ));
    }

    let r_slice = &sig_plain[0..32];
    let s_slice = &sig_plain[32..64];

    let r_bn = BigNum::from_slice(r_slice)?;
    let s_bn = BigNum::from_slice(s_slice)?;

    let sig_obj = openssl::ecdsa::EcdsaSig::from_private_components(r_bn, s_bn)?;

    Ok(sig_obj.to_der()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ec_from_param() {
        let key = JwtAsymmetricKeyInner::from_params_ec(
            ECurve::ECurveP256,
            "GNJoNtvpSo5G-V4Dvn322vTtGo-TcOPN7PtRhZgeGPU=",
            "UWl0nY5DZGUF8vsjysSNdPVbxG_pcIvDlrmJ2CKpobM=",
            Some("F996fAciHB1q5Hu2ElijVrKNtdNzTADYyo31HPCG8PQ="),
            true,
        );
        assert!(key.is_ok());
        let key = key.unwrap();
        assert_eq!(
            Some(JwkRepr {
                crv: Some(ECurve::ECurveP256),
                kty: Some(JKeyType::JTypeEc),
                d: Some("F996fAciHB1q5Hu2ElijVrKNtdNzTADYyo31HPCG8PQ".to_string()),
                x: Some("GNJoNtvpSo5G-V4Dvn322vTtGo-TcOPN7PtRhZgeGPU".to_string()),
                y: Some("UWl0nY5DZGUF8vsjysSNdPVbxG_pcIvDlrmJ2CKpobM".to_string()),
                ..Default::default()
            })
            .as_ref(),
            key.private_key_jwk()
        );

        let key = JwtAsymmetricKeyInner::from_params_ec(
            ECurve::ECurveP256,
            "GNJoNtvpSo5G-V4Dvn322vTtGo-TcOPN7PtRhZgeGPU",
            "UWl0nY5DZGUF8vsjysSNdPVbxG_pcIvDlrmJ2CKpobM",
            Some("F996fAciHB1q5Hu2ElijVrKNtdNzTADYyo31HPCG8PQ"),
            false,
        );
        assert!(key.is_ok());
        let key = key.unwrap();
        assert_eq!(None, key.private_key_jwk());
    }

    #[test]
    fn test_ec_basics_full() {
        let key = JwtAsymmetricKeyInner::generate_ec(ECurve::ECurveP256, false).unwrap();
        assert_eq!(true, key.has_private());
        assert_eq!(false, key.is_exportable());
        assert_eq!(None, key.private_key_jwk());
        let msg = "hello!".as_bytes();
        let sig = key.sign_data(msg).unwrap();
        let ans = key.verify_data(msg, &sig);
        assert!(ans.is_ok());
    }

    #[test]
    fn test_rsa_from_param() {
        let key = JwtAsymmetricKeyInner::from_params_rsa(
            JwtAlgorithm::RS512,
            "nzyis1ZjfNB0bBgKFMSvvkTtwlvBsaJq7S5wA-kzeVOVpVWwkWdVha4s38XM_pa_yr47av7-z3VTmvDRyAHcaT92whREFpLv9cj5lTeJSibyr_Mrm_YtjCZVWgaOYIhwrXwKLqPr_11inWsAkfIytvHWTxZYEcXLgAXFuUuaS3uF9gEiNQwzGTU1v0FqkqTBr4B8nW3HCN47XUu0t8Y0e-lf4s4OxQawWD79J9_5d3Ry0vbV3Am1FtGJiJvOwRsIfVChDpYStTcHTCMqtvWbV6L11BWkpzGXSW4Hv43qa-GSYOD2QU68Mb59oSk2OB-BtOLpJofmbGEGgvmwyCI9Mw",
            "AQAB",
            Some("KIBGrbCSW2O1yOyQW9nvDUkA5EdsS58Q7US7bvM4iWpuDIBwCXur7_VuKnhn_HUhURLzj_JNozynSChqYyG-CvL-ZLy82LUE3ZIBkSdv_vFLFt-VvvRtf1EcsmoqenkZl7aN7HD7DJeXBoz5tyVQKuH17WW0fsi9StGtCcUl-H6KzV9Gif0Kj0uLQbCg3THRvKuueBTwCTdjoP0PwaNADgSWb3hJPeLMm_yII4tIMGbOw-xd9wJRl-ZN9nkNtQMxszFGdKjedB6goYLQuP0WRZx-YtykaVJdM75bDUvsQar49Pc21Fp7UVk_CN11DX_hX3TmTJAUtqYADliVKkTbCQ"),
            Some("y1OPLQcZt4NQnQzPz2SBJqQN2P5u3vXl-zNVKP8w4eBv0vWuJJF-hkGNnSxXQrTkvDOIUddSKOzHHgSg4nY6K02ecyT0PPm_UZvtRpWrnBjcEVtHEJNpbU9pLD5iZ0J9sbzPU_LxPmuAP2Bs8JmTn6aFRspFrP7W0s1Nmk2jsm0"),
            Some("yH0X-jpoqxj4efZfkUrg5GbSEhf-dZglf0tTOA5bVg8IYwtmNk_pniLG_zI7c-GlTc9BBwfMr59EzBq_eFMI7-LgXaVUsM_sS4Ry-yeK6SJx_otIMWtDfqxsLD8CPMCRvecC2Pip4uSgrl0MOebl9XKp57GoaUWRWRHqwV4Y6h8"),
            Some("GYSOJoeKmbZ4So2OEyg48UGLn10nSwIvb9xcZr3IQ8DoJ-C_j0VXTMAzZrNm7C9Jubj59dOBMzOsNb889c8Cs-aU6TyNDo4Fjdd06vPjyr5v-BmtjI4DoD1kJLJbpTv6DHF2z8Fgt4XEXa_bXELZ_SJn_Z9rKS_-hAxSHfh6f-0"),
            Some("CqVZ2JexZyR0TUWf3X80YexzyzIq-OOTWicNzDQ29WLm9xtr2gZ0SUlfd72bGpQoyvDuawkm_UxfwtbIxALkvpg1gcN9s8XWrkviLyPyZF7H3tRWiQlBFEDjnZXa8I7pLkROCmdp3fp17cxTEeAI5feovfzZDH39MdWZuZrdh9E"),
            Some("UxL_Eu5yvMK8SAt_dJK6FedngcM3JEFNplmtLYVLWhkIlNRGDwkg3I5Ky18Ae9n7dHVueyslrb6weq7dTkYDi3iOYRW8HRkIQh06wEdbxt0shTzAJvvCQfrBjg_3747WSsf_zBTcHihTRBdAv6OmdhV4_dD5YBfLAkLrd-mX7iE"),
            true,
        );
        assert!(key.is_ok());
        let key = key.unwrap();
        assert_eq!(
            Some(JwkRepr {
                kty: Some(JKeyType::JTypeRsa),
                n: Some("nzyis1ZjfNB0bBgKFMSvvkTtwlvBsaJq7S5wA-kzeVOVpVWwkWdVha4s38XM_pa_yr47av7-z3VTmvDRyAHcaT92whREFpLv9cj5lTeJSibyr_Mrm_YtjCZVWgaOYIhwrXwKLqPr_11inWsAkfIytvHWTxZYEcXLgAXFuUuaS3uF9gEiNQwzGTU1v0FqkqTBr4B8nW3HCN47XUu0t8Y0e-lf4s4OxQawWD79J9_5d3Ry0vbV3Am1FtGJiJvOwRsIfVChDpYStTcHTCMqtvWbV6L11BWkpzGXSW4Hv43qa-GSYOD2QU68Mb59oSk2OB-BtOLpJofmbGEGgvmwyCI9Mw".to_string()),
                e: Some("AQAB".to_string()),
                d:  Some("KIBGrbCSW2O1yOyQW9nvDUkA5EdsS58Q7US7bvM4iWpuDIBwCXur7_VuKnhn_HUhURLzj_JNozynSChqYyG-CvL-ZLy82LUE3ZIBkSdv_vFLFt-VvvRtf1EcsmoqenkZl7aN7HD7DJeXBoz5tyVQKuH17WW0fsi9StGtCcUl-H6KzV9Gif0Kj0uLQbCg3THRvKuueBTwCTdjoP0PwaNADgSWb3hJPeLMm_yII4tIMGbOw-xd9wJRl-ZN9nkNtQMxszFGdKjedB6goYLQuP0WRZx-YtykaVJdM75bDUvsQar49Pc21Fp7UVk_CN11DX_hX3TmTJAUtqYADliVKkTbCQ".to_string()),
                p:  Some("y1OPLQcZt4NQnQzPz2SBJqQN2P5u3vXl-zNVKP8w4eBv0vWuJJF-hkGNnSxXQrTkvDOIUddSKOzHHgSg4nY6K02ecyT0PPm_UZvtRpWrnBjcEVtHEJNpbU9pLD5iZ0J9sbzPU_LxPmuAP2Bs8JmTn6aFRspFrP7W0s1Nmk2jsm0".to_string()),
                q:  Some("yH0X-jpoqxj4efZfkUrg5GbSEhf-dZglf0tTOA5bVg8IYwtmNk_pniLG_zI7c-GlTc9BBwfMr59EzBq_eFMI7-LgXaVUsM_sS4Ry-yeK6SJx_otIMWtDfqxsLD8CPMCRvecC2Pip4uSgrl0MOebl9XKp57GoaUWRWRHqwV4Y6h8".to_string()),
                dp: Some("GYSOJoeKmbZ4So2OEyg48UGLn10nSwIvb9xcZr3IQ8DoJ-C_j0VXTMAzZrNm7C9Jubj59dOBMzOsNb889c8Cs-aU6TyNDo4Fjdd06vPjyr5v-BmtjI4DoD1kJLJbpTv6DHF2z8Fgt4XEXa_bXELZ_SJn_Z9rKS_-hAxSHfh6f-0".to_string()),
                dq: Some("CqVZ2JexZyR0TUWf3X80YexzyzIq-OOTWicNzDQ29WLm9xtr2gZ0SUlfd72bGpQoyvDuawkm_UxfwtbIxALkvpg1gcN9s8XWrkviLyPyZF7H3tRWiQlBFEDjnZXa8I7pLkROCmdp3fp17cxTEeAI5feovfzZDH39MdWZuZrdh9E".to_string()),
                qi:Some("UxL_Eu5yvMK8SAt_dJK6FedngcM3JEFNplmtLYVLWhkIlNRGDwkg3I5Ky18Ae9n7dHVueyslrb6weq7dTkYDi3iOYRW8HRkIQh06wEdbxt0shTzAJvvCQfrBjg_3747WSsf_zBTcHihTRBdAv6OmdhV4_dD5YBfLAkLrd-mX7iE".to_string()),
                ..Default::default()
            })
            .as_ref(),
            key.private_key_jwk()
        );

        let msg = "hello!".as_bytes();
        let sig = key.sign_data(msg).unwrap();
        let ans = key.verify_data(msg, &sig);
        assert!(ans.is_ok());
    }
}
