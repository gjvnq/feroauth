use crate::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub enum OTP {
    TOTP(BaseOTP),
    HOTP(BaseOTP),
    Recovery(RecoveryCodes),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BaseOTP {
    p_or_c: u32, // period or counter
    secret: Vec<u8>,
    digits: u8,
    alg: HashAlg,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RecoveryCodes {
    hash_alg: HashAlg,
    unused: Vec<String>, // perhaps I should use the database instead of JSON...
                         // also, maybe I should encrypt the clear text codes so the user can still recover them?
}

#[derive(Debug, Deserialize, Serialize)]
pub enum HashAlg {
    Sha1,
    Sha256,
    Sha512,
    Sha3_256,
    Sha3_512,
    Argon2i,
    Argon2d,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Password {
    pre_hash: Option<HashAlg>, //it is ok because we will send this alg before showing the password prompt
    hash_alg: HashAlg,
    hash_val: String,
}

impl Password {
    fn base_verify(&self, _cleartext: &str) -> bool {
        false
    }

    // the clear text can be pre hashed or not hashed at all
    pub fn verify(&self, cleartext: &str) -> bool {
        if self.base_verify(cleartext) {
            return true;
        }
        // pre hash because the browser might not have done it
        let cleartext = "";
        self.base_verify(cleartext)
    }

    pub fn new(cleartext: &str) -> Password {
        Password::new_opt(cleartext, Some(HashAlg::Sha256), HashAlg::Argon2d)
    }

    pub fn new_opt(cleartext: &str, pre_hash: Option<HashAlg>, hash_alg: HashAlg) -> Password {
        let _cleartext = cleartext.trim();
        Password::new_forced(pre_hash, hash_alg, "")
    }

    pub fn new_forced(pre_hash: Option<HashAlg>, hash_alg: HashAlg, hash_val: &str) -> Password {
        Password {
            pre_hash: pre_hash,
            hash_alg: hash_alg,
            hash_val: hash_val.to_string(),
        }
    }
}
