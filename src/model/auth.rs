use crate::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub enum LoginAnswer {
    WrongPassword,
    Wrong2FA,
    Select2FA,
    LoggedIn
}


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
